# mikiwame v0.1 開発指示書

## Explainable Materials Structure Diagnostics in Rust

あなたは、新規Rustライブラリ **`mikiwame`（見極め）** を開発してください。

`mikiwame`は、結晶・周期材料構造を解析し、単一の不透明なスコアではなく、

* 何が構造的に不自然なのか
* どの部位に問題があるのか
* どの数値的根拠からそう判断したのか
* その判断をどこまで信用できるのか
* 入力構造が診断対象として適切か

を説明する、**evidence-firstな材料構造診断ライブラリ**です。

名称は確定済みです。変更提案は不要です。

依存対象の名称は必ず **`chematic`** と記述してください。`schematic`ではありません。

---

## 1. プロダクトの位置づけ

ライブラリ群の関係は次の通りです。

```text
                         chematic
             molecular / periodic structure foundation
                              │
             ┌────────────────┴────────────────┐
             │                                 │
          Molecules                         Materials
             │                                 │
          yomitoki                          mikiwame
 intrinsic molecular                  explainable materials
 synthesizability                     structure diagnostics
             │                                 │
           renkin                            gugen
 retrosynthesis planning           synthesis/process planning
```

役割を混同しないでください。

* `chematic`：構造表現、周期境界、近傍探索、対称性、ファイル入出力などの基盤
* `mikiwame`：構造を診断し、異常・不確実性・適用範囲を説明
* `gugen`：前駆体、工程、温度、雰囲気などの材料合成計画
* `risksieve`：必要に応じたリスク制御・棄却
* `veridict`：候補実装とbaselineの統計的比較

`mikiwame`から`gugen`へ依存してはいけません。将来、`gugen`が`mikiwame`のレポートを任意に利用できる方向にします。

---

## 2. mikiwameが答える問い

`mikiwame`が答える問いは次です。

> この周期材料構造には、どのような構造的異常、入力品質上の問題、配位環境の不自然さ、組成・電荷上の疑義があり、それを示す根拠は何か？

次の問いには答えません。

* この材料は熱力学的に安定か
* formation energyはいくつか
* band gapはいくつか
* 実験で合成できるか
* どの温度・雰囲気で合成するか
* どの前駆体を使うか
* 特許性があるか
* 新素材として有望か
* 臨床・安全用途に利用できるか

これらを推測で補わないでください。

特に、計算していないのに`stable`、`unstable`、`synthesis probability`などの語を使ってはいけません。

mikiwameの中心概念は、**thermodynamic stabilityではなくstructural plausibility and anomaly diagnostics**です。

---

## 3. 永続的なスコープ境界

### mikiwameに含める

* 入力構造品質の検査
* lattice/cellの妥当性検査
* 周期境界下の原子間距離検査
* site重複・異常接近の検出
* 局所配位環境の解析
* 配位多面体の歪み診断
* bond-length distributionの異常診断
* occupancy/disorderの診断
* 組成および形式酸化数の整合性診断
* applicabilityとconfidenceの明示
* 機械可読なfinding code
* 根拠値、対象site、閾値、診断理由
* JSON、テキスト、Markdownレポート
* batch処理
* provenanceとschema version

### mikiwameに含めない

* DFT
* force/energy prediction
* phonon、band structure、DOS
* phase diagram、convex hull
* ML interatomic potential
* 反応経路・工程計画
* 前駆体推薦
* 文献検索
* 材料データベースへのオンライン問い合わせ
* 一般的なCIF基盤の重複実装
* 巨大な学習済みモデルや材料コーパスの同梱
* `pymatgen`全体のRust再実装

---

## 4. chematic依存に関する重要ルール

最初に現在の`chematic` default branchを調査し、以下の存在を確認してください。

* `PeriodicStructure`または同等の型
* `Lattice`
* fractional/Cartesian座標変換
* PBC distance
* periodic neighbor search
* occupancyを持つsite表現
* CIF reader
* symmetry関連API

### 必要な基盤が存在する場合

`mikiwame`はそれを利用してください。

ただし、APIの意味が合わない型を無理に流用してはいけません。例えばMD用のbox型が存在しても、結晶学的なlattice表現として不十分なら、その型へmikiwameを密結合させないでください。

### 必要な基盤が存在しない場合

mikiwame内に大規模な結晶構造基盤を実装してはいけません。

代わりに次を行ってください。

1. mikiwame coreが依存する最小の読み取り専用traitを設計する。
2. テスト用の小さなwire DTOを用意する。
3. `docs/chematic-prerequisites.md`に、chematic側へ必要な型とAPIを具体的に記述する。
4. CIFなどの入出力はadapter層へ隔離する。
5. chematic側の変更は別リポジトリ・別PRとして扱う。
6. 所有者の明示的承認なしに、同じ作業ラウンドでchematicを大規模変更しない。

想定する境界例：

```rust
pub trait PeriodicStructureView {
    fn lattice(&self) -> LatticeView<'_>;
    fn sites(&self) -> &[SiteView];
}

pub struct LatticeView<'a> {
    pub matrix: &'a [[f64; 3]; 3],
}

pub struct SiteView {
    pub element: ElementId,
    pub fractional: [f64; 3],
    pub occupancy: f64,
}
```

これは概念例です。実装前に所有権、allocation、WASM対応、API安定性を検討してください。

mikiwame独自の`Molecule`や巨大な`CrystalStructure`を作って、将来のchematicと競合させてはいけません。

---

## 5. v0.1の完成像

v0.1では「材料全般」を完成させようとしないでください。

対象を次に限定します。

* 三次元周期結晶
* 原子位置とlatticeが明示された構造
* occupancyが有限かつ非負のsite
* 主として無機結晶
* 通常の結晶学的unit cell
* 小分子結晶やMOFは読み込めても、精度保証対象にはまだ含めない
* 表面、界面、アモルファス、ポリマー、巨大欠陥構造は対象外または低applicability

v0.1の価値は、広い物性予測ではなく、

> 壊れた構造、怪しい構造、局所的に不自然な構造を、根拠付きで見つけられる

ことです。

---

## 6. 公開レポート設計

単一の`f64`だけを返してはいけません。

中心となる公開型を、概ね次のように設計してください。

```rust
pub struct MaterialDiagnosticReport {
    pub schema_version: u32,
    pub input: InputSummary,
    pub overall: OverallAssessment,
    pub applicability: ApplicabilityAssessment,
    pub components: Vec<ComponentAssessment>,
    pub findings: Vec<Finding>,
    pub suggestions: Vec<Suggestion>,
    pub provenance: Provenance,
}
```

### OverallAssessment

```rust
pub struct OverallAssessment {
    pub verdict: Verdict,
    pub anomaly_burden: Option<Score01>,
    pub confidence: Score01,
    pub dominant_findings: Vec<FindingCode>,
}
```

想定する`Verdict`：

```rust
pub enum Verdict {
    StructurallyConsistent,
    ReviewRecommended,
    StrongAnomalyDetected,
    OutOfDomain,
    InvalidInput,
}
```

`StructurallyConsistent`は「安定」や「合成可能」を意味しません。実装・README・doc commentですべて明記してください。

### Finding

各findingは最低限、次を持たせてください。

```rust
pub struct Finding {
    pub code: FindingCode,
    pub severity: Severity,
    pub confidence: Score01,
    pub scope: FindingScope,
    pub evidence: Vec<Evidence>,
    pub explanation: String,
    pub limitations: Vec<String>,
}
```

`FindingScope`は例えば次を表現します。

* whole structure
* lattice
* composition
* single site
* site pair
* coordination shell
* polyhedron

### Evidence

文章だけでなく、機械可読な値を残してください。

```rust
pub struct NumericEvidence {
    pub metric: MetricCode,
    pub observed: f64,
    pub expected_range: Option<ClosedRange>,
    pub threshold: Option<f64>,
    pub unit: Option<Unit>,
    pub site_indices: Vec<usize>,
}
```

public resultへNaNや無限大を漏らさないでください。計算不能は`Option`または明示的状態で表現します。

### 分離すべき概念

次を必ず別フィールドにしてください。

* 異常の強さ
* 診断のconfidence
* モデル・ルールのapplicability
* 入力品質
* corpusとの類似性
* 物理的安定性

これらを一つのスコアへ混ぜないでください。

---

## 7. v0.1診断コンポーネント

### 7.1 Input Quality

最優先で実装してください。

検査対象：

* 非有限なlattice値
* 非有限な座標
* singularまたはnear-singular lattice
* 非正のcell volume
* occupancyの負値
* occupancyが許容範囲を超えるsite
* 空構造
* 不明元素
* 完全重複site
* PBC下での異常なsite接近
* 同一元素・同一座標の重複
* Cartesian/fractional変換のround-trip不整合
* metadataと実体の不一致

入力が壊れている場合、後続診断を無理に実行してはいけません。

`InvalidInput`と、実行できなかったcomponentを明示してください。

### 7.2 Lattice and Cell Geometry

検査対象：

* cell volume
* lattice vector lengths
* lattice angles
* extreme aspect ratio
* near-collinear vectors
* unusually acute/obtuse angles
* numerical conditioning
* minimum periodic image distanceに対して不適切に小さいcell

ただし、極端なcell形状が直ちに「誤り」とは限りません。

珍しいことと間違っていることを分け、断定を避けてください。

### 7.3 Site Separation and Collision

PBC下での最近接距離を計算し、次を診断します。

* 原子同士の重複
* 元素半径から見て極端に短い距離
* duplicated periodic images
* site disorderに起因する見かけ上の衝突
* occupancyを考慮すべき衝突

固定の一律距離だけで判定しないでください。

元素半径ベースの期待距離と絶対下限を組み合わせ、判定根拠をレポートへ残します。

### 7.4 Local Coordination Environment

各siteについて、

* coordination number
* neighbor species
* neighbor distances
* shell separation
* local geometry
* coordination environmentの曖昧さ

を解析します。

v0.1では、最初から多数のアルゴリズムを実装しないでください。

一つの説明可能で決定的な近傍定義をbaselineとして実装し、設定とprovenanceへ方法名・cutoff・半径表バージョンを保存します。

近傍定義が不安定なsiteでは、confidenceを下げてください。

### 7.5 Polyhedral Distortion

認識可能な局所配位について、可能な範囲で次を計算します。

* bond-length variance
* quadratic elongation
* angle variance
* central-site displacement
* ideal geometryからの偏差
* coordination-shell asymmetry

どのideal polyhedronと比較したのかを必ず記録してください。

形状認識に自信がない場合は、無理に多面体名を付けず、`AmbiguousCoordinationEnvironment`として扱います。

### 7.6 Composition and Formal Charge Plausibility

形式酸化数候補を用いて、

* charge-neutralな割当が存在するか
* 候補が一意か曖昧か
* 一般的でない酸化状態を必要とするか
* occupancy込みの組成が整合するか

を診断します。

重要事項：

* 酸化数は観測事実ではなく形式的モデルである。
* 一意に決まらない場合は曖昧さを保持する。
* 金属間化合物、混合原子価、非化学量論組成などへ過剰適用しない。
* 割当不能を直ちに「不安定」と表現しない。
* 使用した酸化数表の出典・バージョンをprovenanceへ残す。

### 7.7 Occupancy and Disorder

次を診断します。

* fractional occupancy
* 同一siteにおける複数species
* occupancy sum
* disorderによって距離診断が不確かになる場合
* ordered構造前提の診断が適用不能な場合

disorderをエラー扱いせず、applicabilityとconfidenceへ反映してください。

---

## 8. FindingCode

finding codeは文字列を直接ばらまかず、enumまたは安定したコード体系にしてください。

初期候補：

```text
INPUT_EMPTY_STRUCTURE
INPUT_NONFINITE_COORDINATE
INPUT_INVALID_OCCUPANCY
LATTICE_SINGULAR
LATTICE_POORLY_CONDITIONED
LATTICE_EXTREME_ASPECT_RATIO
SITE_DUPLICATE
SITE_SEVERE_OVERLAP
SITE_UNUSUALLY_SHORT_DISTANCE
COORDINATION_UNDERCOORDINATED
COORDINATION_OVERCOORDINATED
COORDINATION_AMBIGUOUS
POLYHEDRON_BOND_LENGTH_DISTORTION
POLYHEDRON_ANGLE_DISTORTION
COMPOSITION_NO_NEUTRAL_OXIDATION_ASSIGNMENT
COMPOSITION_UNUSUAL_OXIDATION_STATE_REQUIRED
COMPOSITION_OXIDATION_ASSIGNMENT_AMBIGUOUS
DISORDER_PRESENT
APPLICABILITY_OUT_OF_DOMAIN
```

命名は実装前に整理して構いませんが、次を守ってください。

* 意味を後から変更しない
* proseとcodeを分離する
* schema versionを持つ
* severityをcode名へ埋め込まない
* 同じ事象を複数codeで重複報告しない

---

## 9. スコアリング方針

v0.1で精密な「材料正常度スコア」を発明しないでください。

まずは次の方式を採用します。

* componentごとに説明可能なburdenを計算
* overall verdictは重大findingと入力品質から決定
* dominant findingsを明示
* confidenceは独立計算
* applicabilityは独立計算
* 重み付き総和は、科学的根拠と検証が揃うまで中心機能にしない

数値スコアを導入する場合は、次を必須とします。

* 値域を型で保証
* 計算式を文書化
* 各項の寄与をレポート可能
* weightの由来を記録
* test setを見てweightを調整しない
* calibration datasetとholdoutを分離
* scoreを確率として表示しない

---

## 10. Corpus、prototype、類似性

既知材料コーパスを使う機能は、v0.1ではoptionalにしてください。

巨大なデータセットをcrateへ埋め込んではいけません。

外部コーパスを設定した場合のみ、

* nearest prototype
* nearest known structure
* similarity
* novelty
* corpus coverage
* OOD evidence

を計算できる設計にします。

ただし初期段階では、corpus-relativeな信号をoverall verdictへ直接混ぜないでください。

理由：

* 使用コーパスによって結果が変わる
* 未登録と異常は同義ではない
* 新規性と不自然さは同義ではない
* dataset biasが強い

したがってprototype similarityはまず**explanatory evidence only**として扱います。

---

## 11. 競合との差別化

次の既存ツールを模倣するだけでは不十分です。

* pymatgen
* Robocrystallographer
* SMACT
* matminer
* spglib系
* CHGNet / MatGL系

mikiwameの差別化は次です。

1. 単一の診断contractを返す
2. findingが機械可読
3. observed valueとthresholdを残す
4. severity、confidence、applicabilityを分離
5. 入力品質から診断不能まで明示
6. 決定的で再現可能
7. pure Rustを中心とする
8. PythonやWASMへ展開可能な設計
9. ブラックボックス物性予測ではない
10. 「なぜ」を第一級出力にする

Robocrystallographerのような説明生成と競う場合も、自然言語の豊かさより、

> 診断理由の構造化、再現性、数値的根拠、異常度、適用範囲

を優先してください。

---

## 12. 推奨crate構成

最初から過剰にworkspaceを分割しないでください。

v0.1は原則として次で十分です。

```text
mikiwame/
├── Cargo.toml
├── README.md
├── README_ja.md
├── AGENTS.md
├── CHANGELOG.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── docs/
│   ├── architecture.md
│   ├── scientific_scope.md
│   ├── finding_codes.md
│   ├── validation.md
│   ├── competitors.md
│   └── chematic-prerequisites.md
├── src/
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── model.rs
│   ├── finding.rs
│   ├── report.rs
│   ├── provenance.rs
│   ├── structure_view.rs
│   └── diagnostics/
│       ├── mod.rs
│       ├── input_quality.rs
│       ├── lattice.rs
│       ├── separation.rs
│       ├── coordination.rs
│       ├── distortion.rs
│       ├── composition.rs
│       └── disorder.rs
├── src/bin/
│   └── mikiwame.rs
├── tests/
├── fixtures/
├── benchmarks/
└── tasks/
    ├── todo.md
    └── lessons.md
```

CLIが大きくなった場合のみ、後から`mikiwame-cli`へ分離してください。

Python、WASM、MCPはv0.1必須ではありません。Rust coreとJSON contractを先に安定させます。

---

## 13. 公開APIの目標

概念的には次の使い方を目指してください。

```rust
use mikiwame::{analyze, AnalysisConfig};

let config = AnalysisConfig::default();
let report = analyze(&structure, &config)?;

println!("{:?}", report.overall.verdict);

for finding in &report.findings {
    println!(
        "{:?} {:?}: {}",
        finding.severity,
        finding.code,
        finding.explanation
    );
}
```

batch API：

```rust
pub fn analyze_batch<S: PeriodicStructureView>(
    structures: &[S],
    config: &AnalysisConfig,
) -> Vec<Result<MaterialDiagnosticReport, MikiwameError>>;
```

入力順序を維持し、一件の失敗でbatch全体を失敗させない設計を検討してください。

---

## 14. CLI

最低限、次を提供します。

```bash
mikiwame analyze structure.json
mikiwame analyze structure.json --format json
mikiwame analyze structure.json --format markdown

mikiwame batch structures.jsonl \
  --output reports.jsonl

mikiwame explain report.json \
  --finding SITE_SEVERE_OVERLAP

mikiwame doctor
```

CIF adapterが利用可能になったら次を追加します。

```bash
mikiwame analyze material.cif
mikiwame batch ./cifs/*.cif --output reports.jsonl
```

`doctor`は少なくとも次を出力します。

* mikiwame version
* schema version
* chematic version
* enabled features
* radius/oxidation-state table version
* configured corpus
* deterministic mode
* applicable structure classes
* known limitations

---

## 15. テスト戦略

### 15.1 正常構造fixture

少数でもよいので、出典を明記した代表構造を用意します。

候補：

* NaCl / rock salt
* CsCl
* diamond
* zinc blende
* wurtzite
* rutile
* perovskite
* spinel
* graphite

fixtureのライセンスと出典を記録してください。

### 15.2 意図的に壊した構造

正常fixtureから人工的な異常を生成します。

* siteを重複させる
* 一原子だけ極端に移動する
* lattice vectorをほぼ平行にする
* cellを極端に縮める
* occupancyを負にする
* occupancy sumを超過させる
* 一つの配位多面体だけ歪ませる
* 組成をcharge imbalanceへ変更する

期待するfindingが発火し、無関係なfindingが過剰発火しないことを確認してください。

### 15.3 Metamorphic / Invariance Tests

診断結果は原則として次に不変であるべきです。

* site順序の変更
* unit cell内での原点移動
* fractional coordinateへの整数translation
* structure全体の剛体回転
* lattice vector軸の等価な置換
* 同じ構造をsupercell化した場合の正規化済み診断

完全一致が不適切な項目は、何が変わってよいかを文書化してください。

### 15.4 Differential Validation

可能な項目は既存実装と比較します。

* lattice volume
* fractional/Cartesian変換
* PBC distance
* coordination number
* oxidation-state候補
* symmetry情報
* distortion metrics

比較先候補：

* pymatgen
* spglib
* Robocrystallographer
* SMACT

既存実装の出力を盲目的な正解とみなさず、不一致fixtureを保存し、定義差・バグ・曖昧性を分類してください。

### 15.5 Property-Based Tests

少なくとも次を対象に検討してください。

* lattice変換round-trip
* PBC translation invariance
* site permutation invariance
* finite output guarantee
* score range guarantee
* JSON round-trip
* schema backward compatibility

---

## 16. ベンチマークと評価指標

v0.1では「精度98%」のような根拠の薄い単一数字を出さないでください。

次を個別に測定します。

* invalid structure detection rate
* synthetic anomaly detection rate
* known-good structuresに対するfalse-positive rate
* findingごとのcoverage
* finding stability under equivalent representations
* differential agreement
* per-structure runtime
* peak memory
* batch throughput
* panic-free rate
* applicable / out-of-domain比率

評価datasetは次に分けます。

```text
development
validation
holdout
adversarial / malformed
```

holdoutを見てルールやthresholdを変更してはいけません。

benchmarkの各結果には以下を保存します。

* dataset名
* dataset version
* source
* license
* selection rule
* exclusion rule
* mikiwame commit
* chematic version
* configuration
* platform
* timestamp

---

## 17. 科学的主張の規律

README、doc comment、CLI出力で次を守ってください。

### 言ってよい

* structural anomaly detected
* unusual local coordination
* unusually short periodic distance
* formal oxidation-state assignment was not found
* structure is outside the validated domain
* evidence is ambiguous
* review is recommended

### 言ってはいけない

* this material is unstable
* synthesis will fail
* this is a new material
* this material is patentable
* this structure is physically impossible
* this material is safe
* this material has superior performance

強い言葉を使う場合は、測定対象と根拠を限定してください。

---

## 18. 将来のgugen連携

v0.1でgugenを実装しないでください。

ただし、将来gugenが安全に利用できる要約型を検討します。

```rust
pub struct MikiwameHandoff {
    pub schema_version: u32,
    pub verdict: Verdict,
    pub applicability: ApplicabilityLevel,
    pub blocking_findings: Vec<FindingCode>,
    pub caution_findings: Vec<FindingCode>,
    pub provenance_digest: String,
}
```

これは将来契約の概念例です。

gugenは例えば、

* invalid structureならplanningを停止
* severe overlapなら前処理を要求
* oxidation-state ambiguityを計画上の分岐として扱う
* low applicabilityなら警告する

といった利用ができます。

ただしmikiwameは、gugen向けに診断結果を捻じ曲げてはいけません。

---

## 19. Rust品質要件

* Rust 2024 edition
* MSRVは依存する現在のchematicと整合させる
* `#![forbid(unsafe_code)]`
* panicを通常の入力エラー処理に使わない
* typed error
* public APIへNaNを漏らさない
* deterministic by default
* network accessなし
* random処理を導入する場合はseed必須
* `serde`によるschema version付きJSON
* optional featureの依存方向を明確化
* coreは可能な限りWASM互換
* 巨大依存を安易に導入しない
* transitive dependencyとライセンスを監査
* document all public items
* examplesは実際にコンパイルする

最低品質ゲート：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --no-default-features
cargo doc --workspace --all-features --no-deps
cargo audit
```

`cargo doc`には可能なら`RUSTDOCFLAGS="-D warnings"`も適用してください。

---

## 20. 実装フェーズ

### Phase 0 — Landscape and Architecture

コードを書く前に次を行います。

* chematic現状調査
* Rust結晶関連crate調査
* pymatgen / Robocrystallographer / SMACTとの差分整理
* scientific scope文書作成
* public report schema設計
* finding taxonomy設計
* v0.1対象範囲の固定
* chematic prerequisiteの明文化

成果物：

* `docs/architecture.md`
* `docs/scientific_scope.md`
* `docs/competitors.md`
* `docs/chematic-prerequisites.md`
* `tasks/todo.md`

### Phase 1 — Foundation

* crate初期化
* error型
* validated numeric types
* structure view
* report model
* finding model
* provenance
* JSON schema
* README skeleton
* CI

この段階では診断ロジックを増やしすぎないでください。

### Phase 2 — Input Quality and Lattice

* lattice validation
* coordinate validation
* occupancy validation
* overlap/duplicate detection
* PBC distance
* invalid-input short circuit
* malformed fixture

ここを最初の実用release候補とします。

### Phase 3 — Coordination and Distortion

* neighbor definition
* coordination number
* local environment summary
* bond-length variance
* angle variance
* ambiguous-environment handling
* perturbation tests

### Phase 4 — Composition Plausibility

* composition normalization
* oxidation-state candidate enumeration
* charge-neutral assignment
* ambiguity
* unusual-state evidence
* domain limitations

### Phase 5 — CLI and Batch

* analyze
* batch
* explain
* doctor
* JSONL
* Markdown report
* stable exit codes

### Phase 6 — Validation

* known-good fixtures
* synthetic perturbations
* metamorphic tests
* differential checks
* false-positive audit
* benchmark report
* limitations update

### Phase 7 — v0.1 Release Preparation

* README実例を実出力と同期
* changelog
* docs.rs確認
* crates.io package確認
* license確認
* semver/API audit
* release checklist

所有者の明示的許可なくpublishしないでください。

---

## 21. 自律開発ルール

* 可能な作業は自律的に進める。
* 小さく焦点を絞ったcommitを作る。
* unrelated refactorを混ぜない。
* 既存の失敗を見つけた場合、今回変更によるものか既存問題かを切り分ける。
* 科学的根拠が弱いルールを、もっともらしいheuristicとして実装しない。
* thresholdを「常識的だから」で決めない。
* 使える根拠がなければ、findingを延期する。
* 不明点を勝手に決めて製品境界を広げない。
* 承認待ちの事項は後回しにし、独立して進められる作業を継続する。
* 重大なAPI選択、ライセンス問題、別repo変更、外部データ同梱、publishのみstop-and-reportする。
* draft PRを作り、CIがgreenでも勝手にmergeしない。

---

## 22. Stop-and-report条件

次の場合は作業を止め、選択肢・影響・推奨案を報告してください。

* chematic側に必要な周期構造基盤がなく、mikiwameの設計へ重大な影響がある
* 利用候補データのライセンスが不明
* 一般的な酸化数データの再配布条件が不明
* 既存crateとの名前・package衝突
* unsafeまたはC/C++ FFIなしでは重要機能を満たせない
* public schemaの破壊的変更が必要
* 科学的に妥当なthresholdを設定できない
* differential oracle同士が大きく不一致
* false-positive rateが高く、overall verdictが信頼できない
* v0.1の範囲を超えるDFT/ML/合成計画が必要になった

報告時は、単に「できません」と言わず、次を提示してください。

1. 何が判明したか
2. なぜ問題か
3. 最小の解決案
4. 代替案
5. 推奨案
6. 追加作業量
7. 現時点で安全に継続できる作業

---

## 23. 完了条件

v0.1完成と判断できる最低条件は次です。

* Rust libraryとして公開APIがある
* 正常・異常・invalid inputを区別できる
* lattice、occupancy、site overlapを診断できる
* 局所配位またはdistortion診断が最低一つ実用化されている
* findingがmachine-readable
* evidenceに観測値と対象siteが含まれる
* severity、confidence、applicabilityが分離されている
* JSON schema versionがある
* batch処理がある
* CLIがある
* provenanceがある
* known-goodとsynthetic anomalyの評価がある
* false-positive auditがある
* READMEの例が実出力と一致する
* `cargo fmt/clippy/test/doc/audit`が通る
* `mikiwame`が安定性予測や合成可能性を過剰主張していない
* chematicとgugenとの境界が文書化されている

---

## 24. 最終報告形式

作業完了時は、次の順で報告してください。

### 実装内容

追加した機能、公開API、CLI、finding codeを具体的に記載。

### 科学的範囲

何を測定しており、何を測定していないかを明記。

### 検証結果

fixture数、異常検出結果、false positive、differential comparison、runtimeを記載。

### chematicとの関係

利用したAPI、足りなかったAPI、暫定adapter、将来の移行計画を記載。

### 既知の制限

対象外材料、曖昧な配位、酸化数の限界、corpus依存性などを明記。

### 品質確認

実行したコマンドとpass/failを列挙。

### Git状態

branch、commit、PR、working tree、CI状態を記載。

### 次に行うべき一手

機能数を増やす提案ではなく、精度・外部検証・false-positive削減の観点から最大3件に絞る。

---

## 最重要原則

mikiwameの価値は、材料へ「良い・悪い」というラベルを付けることではありません。

> **構造を見極め、疑うべき点とその根拠を、人間と機械の両方が検証できる形で提示すること**

これをすべての設計判断より優先してください。
