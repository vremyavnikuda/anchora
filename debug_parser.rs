// Отладочный скрипт для парсера

use anchora::file_parser::TaskParser;

fn main() {
    let parser = TaskParser::new().unwrap();
    
    let test_lines = vec![
        "// dev:task_1:основная_логика",
        "// dev:task_1:done",
        "// dev:task_1:todo",
        "// dev:task_1: описание задачи",
    ];
    
    for line in test_lines {
        println!("Testing line: '{}'", line);
        match parser.parse_line(line) {
            Some(result) => {
                println!("  Parsed successfully:");
                println!("    section: {}", result.section);
                println!("    task_id: {}", result.task_id);
                println!("    status: {:?}", result.status);
                println!("    description: {:?}", result.description);
                println!("    note: {:?}", result.note);
            }
            None => {
                println!("  Failed to parse");
            }
        }
        println!();
    }
}