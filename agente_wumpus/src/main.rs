use std::io::{self, Write};

use agente_wumpus::agent::ModelAgent;
use agente_wumpus::map::World;

fn print_help() {
    println!("Comandos disponíveis:");
    println!("  f, frente      -> move para frente");
    println!("  l, esquerda    -> gira para esquerda");
    println!("  r, direita     -> gira para direita");
    println!("  t, atirar      -> atira flecha na direção atual");
    println!("  g, pegar       -> pega ouro na casa atual");
    println!("  c, escalar     -> tenta sair da caverna (somente na posição inicial com ouro)");
    println!("  mapa           -> exibe mapa mental do agente");
    println!("  status         -> exibe status atual");
    println!("  ajuda          -> mostra esta ajuda");
    println!("  sair           -> encerra o jogo");
    println!();
}

fn describe_percepts(p: agente_wumpus::agent::Percepts) -> String {
    let mut senses = Vec::new();

    if p.stench {
        senses.push("cheiro");
    }
    if p.breeze {
        senses.push("brisa");
    }
    if p.glitter {
        senses.push("brilho");
    }

    let mut messages = Vec::new();

    if senses.is_empty() {
        messages.push("não sente nada".to_string());
    } else {
        messages.push(format!("sentindo {}", senses.join(" e ")));
    }

    if p.bump {
        messages.push("sentiu um choque".to_string());
    }
    if p.scream {
        messages.push("ouviu um grito".to_string());
    }

    messages.join(". ")
}

fn print_turn(agent: &ModelAgent, percepts: agente_wumpus::agent::Percepts) {
    println!("--------------------------------------------------");
    println!("Status: {}", agent.status_line());
    println!("{}", agent.render_knowledge_map());
    println!("Percepções: {}", describe_percepts(percepts));
}

fn main() {
    // Enunciado: mínimo 16 casas, 1 Wumpus, 3 abismos, 1 ouro.
    let size = 4;
    let pits = 3;
    let wumpus = 1;

    let mut world = World::random(size, pits, wumpus);
    let start = world.start();
    let mut agent = ModelAgent::new(start, size);

    println!("=== Mundo de Wumpus (terminal) ===");
    println!("Você controla o jogador.");
    println!("O agente de IA só mantém o mapa mental (modelo) com as percepções das casas visitadas.");
    println!("Ele registra onde você sentiu brisa, cheiro e brilho do ouro.");
    println!("Posição inicial (base 0): {:?}", start);
    println!("Objetivo: matar Wumpus, pegar ouro e sair pela entrada.");
    println!();
    println!("=== Comandos ===");
    print_help();
    println!("=== Início do jogo ===");

    let mut percepts = agent.sense(&world, false, false);
    agent.update_beliefs(&world, percepts);
    print_turn(&agent, percepts);

    let mut won = false;
    let mut gave_up = false;

    loop {
        if agent.is_dead {
            println!("Você morreu! Pontuação final: {}", agent.score);
            break;
        }

        print!("Comando > ");
        io::stdout().flush().expect("falha ao limpar buffer de saída");

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Falha ao ler comando. Tente novamente.");
            continue;
        }

        let cmd = input.trim().to_lowercase();

        if cmd.is_empty() {
            continue;
        }

        match cmd.as_str() {
            "ajuda" | "help" | "h" => {
                print_help();
                continue;
            }
            "status" => {
                println!("Status: {}", agent.status_line());
                continue;
            }
            "mapa" | "m" => {
                println!("{}", agent.render_knowledge_map());
                continue;
            }
            "sair" | "exit" | "q" => {
                gave_up = true;
                break;
            }

            "f" | "frente" => {
                percepts = agent.move_forward(&world);
            }
            "l" | "esquerda" => {
                percepts = agent.turn_left(&world);
            }
            "r" | "direita" => {
                percepts = agent.turn_right(&world);
            }
            "t" | "atirar" => {
                percepts = agent.shoot(&mut world);
                if percepts.scream {
                    println!("Você ouviu um grito! O Wumpus foi morto (+50).");
                }
            }
            "g" | "pegar" => {
                let had_gold = agent.has_gold;
                percepts = agent.grab_gold(&mut world);
                if !had_gold && agent.has_gold {
                    println!("Você pegou o ouro!");
                } else {
                    println!("Não há ouro para pegar aqui.");
                }
            }
            "c" | "escalar" => {
                let success = agent.climb_out(start);
                percepts = agent.sense(&world, false, false);

                if success {
                    won = true;
                    println!("Você escalou para fora com o ouro (+50)!");
                    break;
                } else {
                    println!("Você só pode sair na posição inicial e carregando o ouro.");
                }
            }
            _ => {
                println!("Comando inválido. Digite 'ajuda' para ver os comandos.");
                continue;
            }
        }

        agent.update_beliefs(&world, percepts);
        print_turn(&agent, percepts);
    }

    println!("================ Fim de jogo ================");
    println!("Status final: {}", agent.status_line());
    println!(
        "Objetivo matar Wumpus: {}",
        if !agent.wumpus_alive {
            "concluído"
        } else {
            "não concluído"
        }
    );
    println!(
        "Objetivo sair com ouro: {}",
        if won { "concluído" } else { "não concluído" }
    );
    if gave_up {
        println!("Jogo encerrado pelo jogador.");
    }
    println!("Pontuação final: {}", agent.score);
}
