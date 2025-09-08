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
}