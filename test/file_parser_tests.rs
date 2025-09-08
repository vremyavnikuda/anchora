use anchora::file_parser::*;
use anchora::task_manager::*;

#[test]
fn test_parser_creation() {
    let parser = TaskParser::new();
    assert!(parser.is_ok());
}

#[test]
fn test_parse_full_definition() {
    let parser = TaskParser::new().unwrap();
    
    let test_cases = vec![
        "// dev:task_1: добавить функционал проверки",
        "//dev:task_1: добавить функционал проверки", // без пробела после //
        "//   dev:task_1:   добавить функционал проверки   ", // с лишними пробелами
    ];
    
    for test_case in test_cases {
        let result = parser.parse_line(test_case);
        assert!(result.is_some(), "Failed to parse: {}", test_case);
        
        let parsed = result.unwrap();
        assert_eq!(parsed.section, "dev");
        assert_eq!(parsed.task_id, "task_1");
        assert!(parsed.description.is_some());
        assert!(parsed.description.unwrap().contains("добавить функционал"));
        assert_eq!(parsed.status, None);
        assert_eq!(parsed.note, None);
    }
}

#[test]
fn test_parse_with_status() {
    let parser = TaskParser::new().unwrap();
    
    let test_cases = vec![
        ("// dev:task_1:todo: описание задачи", TaskStatus::Todo),
        ("// ref:cleanup:in_progress: рефакторинг", TaskStatus::InProgress),
        ("// test:check:done: тестирование завершено", TaskStatus::Done),
        ("// bug:fix_123:blocked: заблокировано", TaskStatus::Blocked),
    ];
    
    for (test_case, expected_status) in test_cases {
        let result = parser.parse_line(test_case);
        assert!(result.is_some(), "Failed to parse: {}", test_case);
        
        let parsed = result.unwrap();
        assert_eq!(parsed.status, Some(expected_status));
        assert!(parsed.description.is_some());
    }
}

#[test]
fn test_parse_simple_reference() {
    let parser = TaskParser::new().unwrap();
    
    let test_cases = vec![
        "// dev:task_1",
        "//dev:task_1", // без пробела
        "//   dev:task_1   ", // с пробелами
        "// feature:user_auth",
        "// bugfix:memory_leak",
    ];
    
    for test_case in test_cases {
        let result = parser.parse_line(test_case);
        assert!(result.is_some(), "Failed to parse: {}", test_case);
        
        let parsed = result.unwrap();
        assert!(parsed.section.len() > 0);
        assert!(parsed.task_id.len() > 0);
        assert_eq!(parsed.description, None);
        assert_eq!(parsed.status, None);
        assert_eq!(parsed.note, None);
    }
}

#[test]
fn test_parse_with_note() {
    let parser = TaskParser::new().unwrap();
    
    let test_cases = vec![
        ("// dev:task_1:важная_заметка", "важная_заметка"),
        ("// ref:cleanup:remove_this_code", "remove_this_code"),
        ("// test:integration:check_api_response", "check_api_response"),
    ];
    
    for (test_case, expected_note) in test_cases {
        let result = parser.parse_line(test_case);
        assert!(result.is_some(), "Failed to parse: {}", test_case);
        
        let parsed = result.unwrap();
        assert_eq!(parsed.note, Some(expected_note.to_string()));
        assert_eq!(parsed.description, None);
        assert_eq!(parsed.status, None);
    }
}

#[test]
fn test_parse_status_update() {
    let parser = TaskParser::new().unwrap();
    
    let test_cases = vec![
        ("// dev:task_1:todo", TaskStatus::Todo),
        ("// dev:task_1:in_progress", TaskStatus::InProgress),
        ("// dev:task_1:inprogress", TaskStatus::InProgress), // альтернативный формат
        ("// dev:task_1:progress", TaskStatus::InProgress), // сокращенный формат
        ("// dev:task_1:done", TaskStatus::Done),
        ("// dev:task_1:completed", TaskStatus::Done), // альтернативный формат
        ("// dev:task_1:complete", TaskStatus::Done), // альтернативный формат
        ("// dev:task_1:blocked", TaskStatus::Blocked),
        ("// dev:task_1:block", TaskStatus::Blocked), // сокращенный формат
    ];
    
    for (test_case, expected_status) in test_cases {
        let result = parser.parse_line(test_case);
        assert!(result.is_some(), "Failed to parse: {}", test_case);
        
        let parsed = result.unwrap();
        assert_eq!(parsed.status, Some(expected_status));
        assert_eq!(parsed.description, None);
        assert_eq!(parsed.note, None);
    }
}

#[test]
fn test_parse_invalid_formats() {
    let parser = TaskParser::new().unwrap();
    
    let invalid_cases = vec![
        "// just a comment",
        "// task_1", // отсутствует раздел
        "// :task_1", // пустой раздел
        "// dev:", // отсутствует task_id
        "// dev::description", // отсутствует task_id
        "/* dev:task_1: comment */", // неправильный тип комментария
        "dev:task_1: no comment prefix",
        "",
        "   ",
        "// dev:task-1: дефис в ID", // дефис в task_id
        "// dev:123task: цифра в начале ID",
    ];
    
    for test_case in invalid_cases {
        let result = parser.parse_line(test_case);
        assert!(result.is_none(), "Should not parse invalid format: {}", test_case);
    }
}

#[test]
fn test_parse_edge_cases() {
    let parser = TaskParser::new().unwrap();
    
    // Тест с очень длинными именами
    let long_section = "a".repeat(50);
    let long_task_id = "b".repeat(50);
    let long_description = "c".repeat(200);
    
    let test_line = format!("// {}:{}: {}", long_section, long_task_id, long_description);
    let result = parser.parse_line(&test_line);
    assert!(result.is_some());
    
    let parsed = result.unwrap();
    assert_eq!(parsed.section, long_section);
    assert_eq!(parsed.task_id, long_task_id);
    assert_eq!(parsed.description, Some(long_description));
}

#[test]
fn test_scan_file() {
    let parser = TaskParser::new().unwrap();
    
    let file_content = r#"
use std::collections::HashMap;

fn main() {
    // dev:task_1: добавить новый функционал проверки на ошибки
    println!("Hello, world!");
    
    // dev:task_1
    let x = 42;
    
    // ref:cleanup_task: провести рефакторинг парсера
    println!("Testing task parser");
    
    // dev:task_2:todo: реализовать автосохранение
    let auto_save = true;
    
    // dev:task_2:основная_логика
    if auto_save {
        println!("Auto save enabled");
    }
    
    // dev:task_1:done
    println!("Task 1 completed");
    
    // This is just a regular comment
    let y = 100;
}
"#;
    
    let results = parser.scan_file("test.rs", file_content).unwrap();
    
    assert_eq!(results.len(), 6, "Should find exactly 6 task labels");
    
    // Проверить номера строк (начинается с 1)
    let line_numbers: Vec<u32> = results.iter().map(|(line, _)| *line).collect();
    assert_eq!(line_numbers, vec![5, 8, 11, 14, 17, 22]);
    
    // Проверить содержимое первой метки
    let (line, label) = &results[0];
    assert_eq!(*line, 5);
    assert_eq!(label.section, "dev");
    assert_eq!(label.task_id, "task_1");
    assert!(label.description.is_some());
}

#[test]
fn test_scan_empty_file() {
    let parser = TaskParser::new().unwrap();
    
    let results = parser.scan_file("empty.rs", "").unwrap();
    assert_eq!(results.len(), 0);
    
    let results = parser.scan_file("whitespace.rs", "   \n\n  \t  \n").unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_scan_file_with_mixed_comments() {
    let parser = TaskParser::new().unwrap();
    
    let file_content = r#"
/* Block comment
   dev:task_1: this should not be parsed
*/

// dev:task_1: this should be parsed
let x = 42; // dev:task_2: inline comment with task

/*
 * Multi-line block comment
 * // dev:task_3: this should not be parsed
 */

// dev:task_4:todo: another valid task
"#;
    
    let results = parser.scan_file("mixed.rs", file_content).unwrap();
    
    assert_eq!(results.len(), 3, "Should only parse line comments");
    
    // Проверить что правильные задачи найдены
    let task_ids: Vec<&String> = results.iter().map(|(_, label)| &label.task_id).collect();
    assert!(task_ids.contains(&&"task_1".to_string()));
    assert!(task_ids.contains(&&"task_2".to_string()));
    assert!(task_ids.contains(&&"task_4".to_string()));
}

#[test]
fn test_update_project_from_labels() {
    let parser = TaskParser::new().unwrap();
    let mut project_data = ProjectData::new(None);
    
    let labels = vec![
        (10, ParsedTaskLabel {
            section: "dev".to_string(),
            task_id: "task_1".to_string(),
            status: None,
            description: Some("Новая задача".to_string()),
            note: None,
        }),
        (15, ParsedTaskLabel {
            section: "dev".to_string(),
            task_id: "task_1".to_string(),
            status: None,
            description: None,
            note: Some("Дополнительная заметка".to_string()),
        }),
        (20, ParsedTaskLabel {
            section: "dev".to_string(),
            task_id: "task_1".to_string(),
            status: Some(TaskStatus::InProgress),
            description: None,
            note: None,
        }),
    ];
    
    let result = parser.update_project_from_labels(
        &mut project_data,
        "test.rs",
        labels
    );
    
    assert!(result.is_ok());
    
    // Проверить что задача создана
    let task = project_data.get_task("dev", "task_1").unwrap();
    assert_eq!(task.title, "Новая задача");
    assert_eq!(task.status, TaskStatus::InProgress); // Должен быть обновлен
    
    // Проверить файловые ассоциации
    assert!(task.files.contains_key("test.rs"));
    let file_info = &task.files["test.rs"];
    assert_eq!(file_info.lines.len(), 3);
    assert!(file_info.lines.contains(&10));
    assert!(file_info.lines.contains(&15));
    assert!(file_info.lines.contains(&20));
    
    // Проверить заметки
    assert_eq!(file_info.notes.get(&15), Some(&"Дополнительная заметка".to_string()));
}

#[test]
fn test_status_parsing_case_insensitive() {
    let parser = TaskParser::new().unwrap();
    
    let test_cases = vec![
        ("// dev:task_1:TODO", TaskStatus::Todo),
        ("// dev:task_1:Todo", TaskStatus::Todo),
        ("// dev:task_1:IN_PROGRESS", TaskStatus::InProgress),
        ("// dev:task_1:DONE", TaskStatus::Done),
        ("// dev:task_1:BLOCKED", TaskStatus::Blocked),
    ];
    
    for (test_case, expected_status) in test_cases {
        let result = parser.parse_line(test_case);
        assert!(result.is_some(), "Failed to parse: {}", test_case);
        
        let parsed = result.unwrap();
        assert_eq!(parsed.status, Some(expected_status));
    }
}

#[test]
fn test_unicode_support() {
    let parser = TaskParser::new().unwrap();
    
    let test_cases = vec![
        "// разработка:задача_1: добавить поддержку юникода",
        "// тест:проверка_api: протестировать ответы сервера",
        "// 开发:任务_1: 添加功能", // Китайские символы
        "// развитие:таск_123: описание с эмодзи 🚀",
    ];
    
    for test_case in test_cases {
        let result = parser.parse_line(test_case);
        assert!(result.is_some(), "Failed to parse Unicode: {}", test_case);
        
        let parsed = result.unwrap();
        assert!(parsed.section.len() > 0);
        assert!(parsed.task_id.len() > 0);
        assert!(parsed.description.is_some());
    }
}