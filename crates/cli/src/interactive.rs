use anyhow::{Context, Result};
use std::io::{self, Write};

use loreforge_llm::OllamaClient;

/// Проверка Ollama и выбор модели пользователем.
pub async fn pick_model(
    client: &OllamaClient,
    current_default: &str,
) -> Result<String> {
    // 1. Проверка Ollama
    let models = match client.ping().await {
        Ok(m) => m,
        Err(_) => {
            print_ollama_help();
            anyhow::bail!("Ollama недоступна");
        }
    };

    if models.is_empty() {
        print_no_models_help();
        anyhow::bail!("Нет моделей");
    }

    // 2. Сортировка по рекомендации
    let sorted = sort_by_recommendation(&models);

    // 3. Показать меню
    println!("\nВыберите модель для извлечения:\n");
    for (i, name) in sorted.iter().enumerate() {
        let size = get_model_size_hint(name, &models);
        let marker = if name == current_default { " ← текущая" } else { "" };
        let recommended = if i == 0 { " (рекомендуемая)" } else { "" };
        println!("  {}. {:40} {}{}{}", i + 1, name, size, marker, recommended);
    }
    println!();

    // 4. Прочитать выбор
    loop {
        print!("Выбор [1-{}, Enter = 1]: ", sorted.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            return Ok(sorted[0].clone());
        }

        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= sorted.len() => {
                return Ok(sorted[n - 1].clone());
            }
            _ => {
                println!("Неверный выбор, попробуй снова");
            }
        }
    }
}

fn sort_by_recommendation(models: &[String]) -> Vec<String> {
    let mut sorted = models.to_vec();

    sorted.sort_by_key(|name| {
        let n = name.to_lowercase();
        // Приоритет:
        // 1. qwen2.5-coder:14b  → 0
        // 2. qwen2.5-coder:7b   → 1
        // 3. qwen2.5-coder:*    → 2
        // 4. qwen3:*            → 3
        // 5. остальное          → 4
        if n.contains("qwen2.5-coder:14b") {
            0
        } else if n.contains("qwen2.5-coder:7b") {
            1
        } else if n.contains("qwen2.5-coder") {
            2
        } else if n.contains("qwen3") {
            3
        } else {
            4
        }
    });

    sorted
}

fn print_ollama_help() {
    eprintln!("\n╔══════════════════════════════════════════════╗");
    eprintln!("║  Ollama не отвечает                          ║");
    eprintln!("╚══════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("Возможные причины:");
    eprintln!("  1. Ollama не установлена");
    eprintln!("     → Скачай: https://ollama.com/download");
    eprintln!("  2. Ollama не запущена");
    eprintln!("     → Запусти Ollama из меню Пуск");
    eprintln!();
    eprintln!("После установки повтори команду.");
}

fn print_no_models_help() {
    eprintln!("\n╔══════════════════════════════════════════════╗");
    eprintln!("║  Моделей не найдено                          ║");
    eprintln!("╚══════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("Установи одну из рекомендуемых:");
    eprintln!("  ollama pull qwen2.5-coder:7b     (быстрая, ~5 ГБ)");
    eprintln!("  ollama pull qwen2.5-coder:14b    (рекомендуется, ~10 ГБ)");
    eprintln!("  ollama pull qwen3:8b-64k         (длинные тексты, ~5 ГБ)");
    eprintln!();
}

fn get_model_size_hint(_name: &str, _models: &[String]) -> String {
    // TODO: получить размер из /api/tags ответа
    String::new()
}