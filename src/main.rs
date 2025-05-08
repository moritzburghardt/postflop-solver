use postflop_solver::*;
use std::fs;
use serde_json::json;

fn main() {
    let input_text:String = fs::read_to_string("input.json").unwrap();
    let config: serde_json::Value = serde_json::from_str(&input_text).unwrap();

    let turn = config.get("turn").and_then(|v| v.as_str()).and_then(|s| card_from_str(s).ok()).unwrap_or(NOT_DEALT);
    let river = config.get("river").and_then(|v| v.as_str()).and_then(|s| card_from_str(s).ok()).unwrap_or(NOT_DEALT);
    let initial_state = if turn == NOT_DEALT { BoardState::Flop } else { if river == NOT_DEALT { BoardState::Turn } else { BoardState::River } };

    let card_config = CardConfig {
        range: [config["oop_range"].as_str().unwrap().parse().unwrap(), config["ip_range"].as_str().unwrap().parse().unwrap()],
        flop: flop_from_str(config["flop"].as_str().unwrap()).unwrap(),
        turn: turn,
        river: river,
    };

    let bet_sizes = BetSizeOptions::try_from((config["bet_size"].as_str().unwrap(), config["raise_size"].as_str().unwrap())).unwrap();

    let tree_config = TreeConfig {
        initial_state: initial_state,
        starting_pot: config["starting_pot"].as_i64().unwrap() as i32,
        effective_stack: config["effective_stack"].as_i64().unwrap() as i32,
        rake_rate: 0.0,
        rake_cap: 0.0,
        flop_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()], // [OOP, IP]
        turn_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
        river_bet_sizes: [bet_sizes.clone(), bet_sizes],
        turn_donk_sizes: None, // use default bet sizes
        river_donk_sizes: Some(DonkSizeOptions::try_from("50%").unwrap()),
        add_allin_threshold: 1.5, // add all-in if (maximum bet size) <= 1.5x pot
        force_allin_threshold: 0.15, // force all-in if (SPR after the opponent's call) <= 0.15
        merging_threshold: 0.1,
    };

    let action_tree = ActionTree::new(tree_config).unwrap();
    let mut game = PostFlopGame::with_config(card_config, action_tree).unwrap();

    game.allocate_memory(false);

    // solve the game
    let max_num_iterations = 1000;
    let target_exploitability = game.tree_config().starting_pot as f32 * 0.005; // 0.5% of the pot
    let _exploitability = solve(&mut game, max_num_iterations, target_exploitability, true);
    game.cache_normalized_weights();
    fn traverse_node(game: &mut PostFlopGame, history: &mut Vec<usize>) -> serde_json::Value {
        if game.is_chance_node() || game.is_terminal_node() {
            return json!(null);
        }

        let actions: Vec<Action> = game.available_actions();
        let strategy = game.strategy();
        let player = game.current_player();

        let mut children = Vec::new();

        for (i, _) in actions.iter().enumerate() {
            history.push(i);
            game.apply_history(&history);
            let child = traverse_node(game, history);
            children.push(child);
            history.pop();
            game.apply_history(&history);
        }

        json!({
            "actions": actions,
            "strategy": strategy,
            "player": player,
            "children": children
        })
    }

    let mut history: Vec<usize> = Vec::new();

    let output = json!({
        "equity_0": game.equity(0),
        "equity_1": game.equity(1),
        "expected_values_0": game.expected_values(0),
        "expected_values_1": game.expected_values(1),
        "initial_state": game.tree_config().initial_state,
        "hands_0": holes_to_strings(game.private_cards(0)).unwrap(),
        "hands_1": holes_to_strings(game.private_cards(1)).unwrap(),
        "nodes": traverse_node(&mut game, &mut history),
    });

    let mut file:fs::File = fs::File::create("output.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &output).unwrap();
}
