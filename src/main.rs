/*
1. 初期集団の生成: ランダムに個体を生成し、初期集団を形成する。
   - ※空間の次元を n とする。
   - 各個体が持つべき情報
       - 空間座標（行動記述子）: n 次元ベクトル
       - 新規性スコア: 実数値（例: `f64`）
   - 空間（環境）が持つべき情報
       - アーカイブ: 過去に「新規」と判断された n 次元ベクトルの集合

2. 評価関数の適用: 各個体 x に対して、
   - 現在の対象の個体の座標と、アーカイブおよび現在の集団の他の各個体の座標との距離を計算し、
   - その中から k 個の最近傍を取り、その平均距離を新規性スコアとする。
       - 距離の計算にはユークリッド距離などのメトリックを用いる。
       - このスコアが大きいほど（近くにある他の個体が少ないほど）、新規性が高いと判断される。

3. アーカイブの更新:
   - 新規性スコアがあらかじめ定めた閾値を超えた個体の座標を、アーカイブに追加する。

4. 選択:
   - 新規性スコアに基づいて、次世代に進む個体を選択する。

5. 交叉と突然変異:
   - 選択された個体を用いて交叉と突然変異を行い、新しい個体を生成する。

6. 規定世代数 / 計算予算 に達するまで、ステップ 2 〜 5 を繰り返す。
*/

use std::vec;

// ユークリッド距離だけは関数に切り出す（さすがにここだけ）
fn distance(a: &[f64], b: &[f64]) -> f64 {
    let mut sum = 0.0;
    for i in 0..a.len() {
        let d = a[i] - b[i];
        sum += d * d;
    }
    sum.sqrt()
}

fn main() {
    // ==== パラメータ ====
    let k: usize = 5; // 近傍として見る個体数
    let threshold: f64 = 1.5; // アーカイブ追加の閾値

    // ==== 初期集団（2次元空間上の4点）====
    let mut population: Vec<Vec<f64>> = vec![
        vec![0.0, 0.0, 0.0, 0.1],
        vec![1.0, 0.0, 0.0, 0.1],
        vec![0.0, 1.0, 0.0, 0.1],
        vec![3.0, 3.0, 0.0, 0.1],
        vec![0.5, 0.5, 0.0, 0.1],
        vec![2.0, 2.0, 0.0, 0.1],
        vec![4.0, 4.0, 0.0, 0.1],
        vec![5.0, 5.0, 0.0, 0.1],
        vec![6.0, 6.0, 0.0, 0.1],
        vec![7.0, 7.0, 0.0, 0.1],
        vec![8.0, 8.0, 0.0, 0.1],
        vec![9.0, 9.0, 0.0, 0.1],
    ];

    // ==== アーカイブ（最初は空）====
    let mut archive: Vec<Vec<f64>> = Vec::new();

    // ==== 現在の他の個体との距離を各格納する集合 ====
    let mut distances: Vec<f64>;
    // ==== 他の現在評価中の個体に近い順に k 個を見たときのそれぞれの距離の合計を格納する変数 ====
    let mut kk_distance_sum: f64;
    // ==== agentの点数順にソートした現世代のpopulation ====
    // (agent, novelty_score)
    let mut scored_population: Vec<(Vec<f64>, f64)>;
    // ==== 選択された次世代個体群 ====
    let mut selected_population: Vec<Vec<f64>>;
    let remain_agents = 5; // 次世代に残す個体数
    // ==== 生成された次世代個体群 ====
    let mut next_population: Vec<Vec<f64>>;
    // ==== 子供の個体 ====
    let mut child_agent: Vec<f64>;

    println!("--- Novelty Evaluation ---");

    for _generation in 0..100 {
        println!("\nGeneration {}", _generation);
        scored_population = Vec::new(); // 次世代個体群を初期化
        // === 各個体について新規性スコアを計算 === //
        // agent ... 現在評価中の個体
        // other_agent ... 比較対象の個体
        for (agent_index, agent) in population.iter().enumerate() {
            distances = Vec::new(); // 距離リストを初期化

            // 1. アーカイブとの距離
            // "アーカイブに保存されている全個体"の座標との距離を計算
            for other_agent in &archive {
                let d = distance(agent, other_agent);
                distances.push(d);
            }

            // 2. "現在の集団"の「他の個体」との距離
            // 全個体との距離を計算してリストに追加
            for (other_agent_index, other_agent) in population.iter().enumerate() {
                if agent_index == other_agent_index {
                    // 自分自身はスキップ
                    continue;
                } else {
                    // 他の個体との距離を計算
                    let d = distance(agent, other_agent);
                    distances.push(d);
                }
            }

            // 3. 現在評価中の個体との距離を小さい順にソート
            distances.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // 4. 近い順に k 個を見て、その平均距離をスコアとする
            let kk = k.min(distances.len());
            kk_distance_sum = 0.0;
            for distance in &distances[0..kk] {
                kk_distance_sum += distance;
            }
            // 新規性スコア（平均距離）
            let novelty = kk_distance_sum / kk as f64;
            // 5. 閾値を超えたらアーカイブに追加
            let added = if novelty > threshold {
                archive.push(agent.clone());
                true
            } else {
                false
            };

            // 5. 次世代個体群に (個体, 新規性スコア) の組を追加
            scored_population.push((agent.clone(), novelty));

            println!(
                "agent = {:?}, novelty = {:.3}, added_to_archive = {:?}",
                agent, novelty, added
            );
        }
        // === 選択 (エリート選択) === //
        // remain_agents文だけ新規性スコアの高い個体を選択
        scored_population.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // 新規性スコアの高い順にソート
        selected_population = scored_population
            .iter()
            .take(remain_agents)
            .map(|(agent, _score)| agent.clone())
            .collect();

        // === 交叉と突然変異 === //
        next_population = Vec::new(); // 生成された次世代個体群を初期化
        for _ in 0..(population.len() - remain_agents) {
            // 交叉: ランダムに2個体を選んで平均を取る
            let parent1 = &selected_population[rand::random_range(0..selected_population.len())];
            let parent2 = &selected_population[rand::random_range(0..selected_population.len())];
            child_agent = Vec::new();
            for i in 0..parent1.len() {
                let mixed_agent = (parent1[i] + parent2[i]) / 2.0;
                child_agent.push(mixed_agent);
            }
            // 突然変異: 各次元に小さなランダムノイズを加える
            for dimension in &mut child_agent {
                let noise: f64 = (rand::random::<f64>() - 0.5) * 0.2; // -0.1 〜 +0.1 のノイズ
                *dimension += noise;
            }
            next_population.push(child_agent);
        }
        // 次世代個体群を更新
        population.clear();
        population.extend(selected_population);
        population.extend(next_population);
    }

    println!("\nArchive size: {}", archive.len());
    println!("Archive contents: {:?}", archive);
}
