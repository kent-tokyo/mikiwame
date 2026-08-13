# mikiwame (見極め)

周期結晶構造のためのExplainable(説明可能な)材料構造診断ライブラリ（Rust製）。

`mikiwame`は3次元周期結晶構造を受け取り、単一の不透明なスコアではなく、**何が**構造的に
不自然か、**どこに**その問題があるか、**どのような根拠**でそう判断したのかを説明します。
何を測定し、何を測定していないかの詳細は
[`docs/scientific_scope.md`](docs/scientific_scope.md)を参照してください。

**ステータス: v0.1、開発初期段階。** 経験的なしきい値を発明せずに実装できる診断のみが
実装済みです。何が未実装で、なぜかは[`tasks/todo.md`](tasks/todo.md)を、設計は
[`docs/architecture.md`](docs/architecture.md)を参照してください。

## mikiwameが主張しないこと

`mikiwame`は熱力学的安定性、生成エネルギー、バンド構造、合成可能性を予測しません。実際に
計算していないのに"unstable"（不安定）や"will synthesize"（合成できる）といった言葉を
使うこともありません（それはスコープ外です。詳細は
[`docs/scientific_scope.md`](docs/scientific_scope.md)）。`Verdict::StructurallyConsistent`
は、実行した診断で構造的異常が見つからなかったことを意味するのみで、安定性や合成可能性の
主張ではありません。

## 使い方

```rust
use mikiwame::{analyze, AnalysisConfig, OwnedStructure, Site};

let config = AnalysisConfig::default();
let report = analyze(&structure, &config); // structure: impl PeriodicStructureView

println!("{:?}", report.overall.verdict);
for finding in &report.findings {
    println!("{:?} {}: {}", finding.severity, finding.code, finding.explanation);
}
```

[`examples/basic.rs`](examples/basic.rs)（`cargo run --example basic`）による実際の出力。
清浄な岩塩型（NaCl）fixtureと、1つのsiteを別のsite上へ移動させたコピーを比較しています。

```text
clean NaCl: StructurallyConsistent
duplicated-site NaCl: StrongAnomalyDetected
  Critical SITE_DUPLICATE: sites 0 and 1 (both Na) coincide under periodic boundary conditions (separation 0.000e0 Å)
```

`analyze`は[`PeriodicStructureView`](src/structure_view.rs)を実装した任意の型を受け取り
ます。自前の構造体、または直接構築用の[`OwnedStructure`](src/structure_view.rs)が使えます。
CIF読み込みはまだありません（下記参照）。

## CLI

```bash
cargo run --bin mikiwame -- analyze structure.json --format markdown
cargo run --bin mikiwame -- analyze structure.json --format json   # デフォルト
cargo run --bin mikiwame -- batch structures.jsonl --output reports.jsonl
cargo run --bin mikiwame -- explain report.json --finding SITE_DUPLICATE
cargo run --bin mikiwame -- doctor
```

`structure.json`（および`structures.jsonl`の各行）は`{"lattice": [[..],[..],[..]],
"sites": [{"element": "Na", "fractional": [0.0,0.0,0.0], "occupancy": 1.0}, ...]}`という
形式です。詳細は[`src/bin/mikiwame.rs`](src/bin/mikiwame.rs)のモジュールdocコメントを
参照してください。これはCLI専用のスキーマであり、レポートの`schema_version`とは独立して
います（まだCIFリーダーがないため、現状これが唯一サポートされているファイル入力です）。
CLIは`cli` Cargo feature（デフォルトで有効）の裏にあります。`cargo build
--no-default-features`とすると`serde_json`を含まない純粋なライブラリビルドになります。

## `chematic`との関係

`mikiwame`は自前の最小限の読み取り専用トレイト`PeriodicStructureView`のみに依存します。
`chematic`の現在のdefault branchには周期構造・occupancyを扱う型が存在しないためです。
調査内容と、より密な統合のために`chematic`側に必要となる型については
[`docs/chematic-prerequisites.md`](docs/chematic-prerequisites.md)を参照してください。

## 未実装

CIF/ファイルI/O、配位環境・歪み・組成の診断（disorderのしきい値不要サブセットである
`DISORDER_PRESENT`、`DISORDER_OCCUPANCY_SUM_EXCEEDS_ONE`は実装済み）、および根拠のない
定数を必要とするしきい値ベースの診断（元素半径、"極端な"格子アスペクト比、酸化数テーブル
など）。詳細は[`tasks/todo.md`](tasks/todo.md)を参照してください。

## 品質ゲート

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
cargo audit
```

## ライセンス

MIT または Apache-2.0 のデュアルライセンス（いずれかを選択可能）。
