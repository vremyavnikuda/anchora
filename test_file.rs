fn main() {
    println!("Hello, world!");
    
    let x = 42;
    
    // ref:cleanup_task: провести рефакторинг парсера
    println!("Testing task parser");
    
    let auto_save = true;
    
    // dev:task_2:основная_логика
    if auto_save {
        println!("Auto save enabled");
    }
    
    // dev:task_1:done
    println!("Task 1 completed");
}