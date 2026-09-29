# Food solver

食材质量和营养素质量的内部单位为 **g**，能量为 **J**；`KCAL = 4184`。

## 需求计算

`dog(age=3, weight=7, active=False, daily_kcal=None)` 当前只支持成年犬维持期。
年龄以年、体重以 kg 输入；不支持生长期、妊娠或哺乳期。一岁以上仍在
生长的大型犬也不应使用这个成年犬模型。

营养表统一采用 [FEDIAF 2025 Table III-3b（PDF 第 16 页）](https://europeanpetfood.org/wp-content/uploads/2025/09/FEDIAF-Nutritional-Guidelines_2025-ONLINE.pdf)
中每 1000 kcal 代谢能的成年犬推荐量，已录入全部有数值的成年犬最低推荐量：

- `active=False`：95 kcal/kg^0.75 的参考档；`True`：110 档，表示普通活动，非工作犬。
- 默认每日 kcal = 对应档系数 × 体重^0.75，作为起始估计。
- 每日营养量 = 对应档每 1000 kcal 的推荐量 × 每日 kcal / 1000。
- `daily_kcal` 可覆盖能量估计；营养浓度仍由 `active` 选择。两者都应按实际情况设定。
- 硒使用湿粮行。A、D、E 从 IU 转换；E 使用天然 RRR-alpha-tocopherol 的
  0.67 mg/IU 当量，不再把微克误用为毫克。
- 上限仅录入此表按能量列出的营养上限 (N)。旧碘、硒、钠上限不沿用：
  干物质基准的法律限值 (L) 不能直接充当每 1000 kcal 上限。
  `None` 表示模型未设上限，并非无限摄入安全。

保留原有 HARD / SOFT / NOT_REQUIRED 设置。`optimal` 只表示优化问题求解成功，
软目标仍可不足或超过参考范围；数据库缺失值也仍按零参与现有计算。
这不是完整的营养充分性验证。

成年犬命令另外启用钙磷质量比的硬约束 `1 <= Ca/P <= 2`，即
`Ca >= P` 和 `Ca <= 2P`；钙和磷各自的绝对需求仍单独约束。
比例不随批次天数缩放。它实现为 `(P-Ca) @ x <= 0` 和
`(Ca-2*P) @ x <= 0` 两行线性约束（这里 Ca、P 是食材营养密度行，
不是二次目标矩阵）；磷的正下限保证分母非零。该约束由 `PlanRequest`
固定附带，不需要调用方显式添加。

## 目标函数

只有下限 L 的软需求使用 `max(0, (L - 营养供给) / L)^2`：只惩罚不足，
达到或超过最低量后惩罚为零，不再使用下限的 1.05 倍作为目标。
通过引入非负缺口变量 s，并约束 `s >= 1 - 营养供给/L`，最小化 `s^2`，
仍可用凸二次规划求解。零下限跳过惩罚，避免除零。

有上下限的软需求使用
`max(0, (L - 供给)/L)^2 + max(0, (供给 - U)/U)^2`：
不足除以下限 L，超量除以上限 U，区间内（包括端点）惩罚为零。
每一侧各引入一个非负辅助变量，其平方进入目标，约束仍为线性。
10% 不足和 10% 超量具有相同惩罚，不代表两者的健康风险相同。
零下限省略不足项；上限必须为有限正数且不小于下限。
硬约束和钙磷比保持不变。

当软需求只有正下限、上限为 `None` 时，默认增加一个隐式软上限
`U = 1.5 * L`。超过 U 后按 `max(0, 供给/U - 1)^2` 惩罚；它只影响可行方案
之间的选择，不会造成无解。倍数由 `PlanRequest::implicit_soft_upper_multiplier`
配置：调小更贴近最低量，调大更宽松，设为 `None` 则关闭。**硬需求不生成隐式
上限**：没有显式上限的硬需求是真正开放式的，不会因超量被惩罚。

显式上限始终优先。最低量为 0 时不会自动生成上限。这个倍数是配方偏好，
不是 FEDIAF 的安全上限，也不能据此判断某营养素超过后是否有毒性。

营养报告显示实际参与优化的区间。颜色只由状态决定：红色表示低于最低值，
黄色表示高于显式上限，浅绿色表示位于区间内；NOT_REQUIRED 项额外加删除线。
隐式软上限只是偏好，超过它不会显示为 `above maximum`，其上限值以下划线标出。
黄色超量不等于毒性或硬约束失败。
`--detail true` 的食材分项、营养总量和范围都按每天显示，即使求解的是多日批次。
实现把这些公式手工转成 Clarabel 的标准形 `minimize (1/2) z'Pz + q'z`，
`A z <= b`：每个软损失的一侧各一个辅助变量，其平方进 `P` 的对角线，
约束仍为线性行。推导细节见 `rust/food-core/src/recipe.rs` 的模块文档。

`Solution` 只在最优时给出 `grams`；不可行或不收敛时为 `None`，不会返回旧配方。
失败状态通过 `SolveStatus` 原样带出。求解器异常会正常抛出。

Clarabel 文档：<https://github.com/oxfordcontrol/Clarabel.rs>。

启用 `minimize_usage` 时增加 `0.05 * (食材克数 / 食材上限)^2`，避免原来的
克数平方惩罚随批量大小放大。库存和需求一起乘 10 时，目标函数保持一致。

## 使用与验证

```sh
just build
just test
just run solve -d 10 -i PORK:500 -i RICE:700
```

需要指定已确认的每日热量时可使用 `--daily-kcal`，例如 `solve -d 10
--daily-kcal 500`；这个示例数值不是个体喂养建议。

每日需求指全天总量，尚未扣除商品粮贡献。不带后缀的 `-i FOOD:GRAMS` 是上限，
求解器可以少用；要固定用量需显式写 `:fixed`。给定食材在新的默认需求下可能
无解；不要仅为得到解而提高热量。

## 新增营养目标与数据覆盖

新增 17 个 REQUIRED + SOFT 目标：总脂肪、亚油酸、10 种必需氨基酸、
蛋氨酸+胱氨酸、苯丙氨酸+酪氨酸、B5、叶酸和氯。组合目标按各食材的
分项含量相加；单独的蛋氨酸和苯丙氨酸目标也保留。B5 使用
`VITAMIN_B5`，旧数据中的 `PANTOTHENIC_ACID` 只作为缺少前者时的备用值，
不重复计入。

表中 ALA、花生四烯酸、EPA+DHA、生物素、维生素 K 没有成年犬数值最低量，
因此没有套用幼犬数值，也没有增加零值目标。脂肪酸数据类型支持记录这些
分项；USDA 仅映射明确标明脂肪酸身份的条目，不能用总 PUFA 或不明确的
18:2、18:3 代替亚油酸或 ALA。

已有 A/D/E、胆碱和硒的 NOT_REQUIRED 设置保持不变；录入完整需求表不等于
所有行都参与优化，也不等于食物数据完整。新增的最低需求使用单边平方
缺口损失，不惩罚超过最低量；其他偏好仍可与这些软需求权衡。

旧 JSON 缓存不会自动多出以前被丢弃的氨基酸/脂肪酸。可以显式重新获取：

```sh
just refresh-foods RICE EGG CHICKEN_BREAST   # 抓取指定食材（food fetch）
just refresh-foods                           # 全部可抓取食材
just refresh-foods --list                    # 列出食材、可用来源与当前选择
```

这个命令会联网并覆盖指定食材缓存；来源本身未提供的字段仍会缺失。
不要把其他食材的数据或总蛋白估算值填作已测量的氨基酸含量。
报告中的 `incomplete data: ...` 列出实际使用但该营养素数据不全的食材。
优化仍使用已知贡献（未知项按零贡献处理），组合项只有一项已知时使用
部分和并标记不完整；不再通过读取 defaultdict 把缺失字段写成“已知零”。

## 代码结构：Rust 库 + CLI + MCP

求解器现在是 Rust 工作区，规划逻辑在一个库里，CLI 和 MCP 是它的两个前端：

- `rust/food-base/`：`Nutrient` 枚举、`Food` 结构体与 `nutrient_value`，以及
  字段名表 `NUTRIENT_FIELDS`。它单独成 crate 是因为构建脚本不能依赖它正在
  构建的那个 crate：`food-core` 的 `build.rs` 需要这份字段表来生成内嵌数据，
  把共享词汇放在两者之下就避免了解析源码。字段表与枚举的一致性由
  `rust/food-base/tests/fields.rs` 锁住。
- `rust/food-core/`：`PlanRequest`（食材 + 天数 + 犬只档案）进，`PlanResult`
  （配方 + 每日营养报告）出。`IngredientSpec` 用上下界描述食材：相等即固定
  用量，`0 ~ 上限` 表示可选。数量都是**整批克数**，`days` 只缩放需求。
- `rust/food-cli/`：`food` 命令。
- `rust/food-mcp/`：`food-mcp`，把同一流程暴露成 `list_foods`、`get_needs`、
  `solve_recipe`、`build_info` 四个 MCP 工具。`solve_recipe` 与 `get_needs` 的
  `weight`、`days`、`age` 是必填项（没有默认值），犬只档案未知时应先询问用户。
- 构建信息：`food-core` 的 `build.rs` 把构建时刻（ISO 8601 UTC）与 git 短
  修订号编进二进制，导出为 `food_core::BUILD_INFO`。CLI 的每个子命令都在
  输出开头打印这一行，MCP 的 `build_info` 工具返回同样的内容，便于把一份
  配方或日志对应回具体版本。`SOURCE_DATE_EPOCH` 可覆盖时间以支持可复现构建。
- `rust/food-cli/` 的 `fetch` 子命令（`food fetch`）：抓取 USDA portal、
  《中国食物成分表》、USDA SR Legacy 与日本 MEXT 四种来源，各自写进
  `foods/{FOOD}_{Source}.json`；`expand-sr-legacy` 子命令把 SR Legacy 的
  全量 JSON 展开成同样的逐食材文件。
- **来源之间不做合并**：同一食材的多个来源是同一食物的不同采样版本，各自
  独立成文件，`CHOOSE` 只选其中**一行**。SR Legacy 的行通常面板最全
  （氨基酸、脂肪酸、胆碱、维生素 K、B12），所以 28 个食材的 `CHOOSE` 指向
  它；只被 SR Legacy 收录的食材（如 `EGG_YOLK`）则直接以它为默认来源。
- 缓存布局：同一种食材每个来源一个文件（如 `BAICAI_Chinanutri.json`、
  `BEEF_SrLegacy.json`），文件体只存营养素，来源由文件名承载。`build.rs`
  把**全部**来源都编进二进制，运行期由 `src/foods.rs` 的 `CHOOSE` 表决定用
  哪一份——CLI 和 MCP 传入的始终只是食材名，名字 + 来源在内部合成后再查表。
  换来源只改 `CHOOSE` 一行并重建，不必重新抓取。
- `rust/food-core/build.rs`：构建时读取 `foods/{FOOD}_{Source}.json`（外加
  `build.rs` 里几条内联配方）生成内嵌食材营养表，`foods.rs` 再用 `include!`
  引进来。字段名取自 `food-base::NUTRIENT_FIELDS`；生成的 `Food { .. }`
  字面量会写出每个字段，所以字段表与结构体一旦不一致就会编译失败。

求解器用 [Clarabel](https://github.com/oxfordcontrol/Clarabel.rs)（牛津大学，
CVXPY 也用它）。目标函数：只有下限的软需求用 `max(0, (L - y) / L)^2`，有上下
限的用两侧相对平方损失，另加 `U = 1.5L` 的隐式软上限；硬需求与钙磷比是硬约束。
参考基线记录在 `rust/food-core/tests/reference.rs`，语义或 `CHOOSE` 变化时需重录。

```sh
just build          # cargo build --release
just test           # cargo test --workspace
just run solve -d 10 -i PORK:500 -i RICE:700:minimize -i EGG:700:minimize -i EGG_SHELL_POWDER:16:minimize -i SALT:2:minimize  # 运行 Rust CLI
just check          # cargo check
just clippy         # cargo clippy（-D warnings）
```

CLI 支持 `-d/--day`、`--detail`、`--daily-kcal`、`--weight`、`--age`、
`--active`，以及可重复的 `-i FOOD:GRAMS[:optional|minimize|fixed]`。不带后缀时
GRAMS 是上限（求解器可少用），要固定用量需显式 `:fixed`。
求解时必须至少给一个 `-i`（不再有内置默认批次）。`food report -i FOOD:GRAMS:fixed`
按**给定用量**（不求解）直接输出同一套营养报告，用来看某批配方实际提供了多少
营养。`food fetch --list` 列出全部食材，`food fetch` 刷新食材缓存。

## 数据来源

食材数据有两个入口：

- `just refresh-foods`（`food fetch`）按 `foods/{FOOD}_{Source}.json` 逐食材抓取
  USDA portal、《中国食物成分表》、USDA SR Legacy 与日本 MEXT 四种来源；
  `food fetch --list` 列出全部食材及其可用来源。
- `just run expand-sr-legacy --json <全量 JSON>` 把 SR Legacy 的批量 JSON
  展开成同样的逐食材文件。

同一种食材每个来源一个文件，`build.rs` 把全部来源编进二进制，运行期由
`rust/food-core/src/foods.rs` 的 `CHOOSE` 表决定用哪一份；换来源只改 `CHOOSE`
一行并重建，不必重新抓取。`refresh-foods` 需要 USDA 的 `USDA_API_KEY`，
抓取中国营养网还需要能访问它的网络（可用 `HTTPS_PROXY`）。

## 作为 Codex 插件使用

`just make-plugin` 会把本仓库打包成**自包含**的 Codex 插件 `food-solver`
（默认写到 `~/plugins/food-solver`，并更新 `~/.agents/plugins/marketplace.json`）：

```sh
just make-plugin                 # 构建并更新个人 marketplace
just make-plugin --install       # 再执行 codex plugin add
just make-plugin --skip-build --dest dist --archive dist/food-solver.tar.gz
just release-plugin              # musl 静态构建 + 打包 dist/food-solver-musl.tar.gz
```

插件目录约 5 MB：`bin/food-mcp`、`bin/food`、`.mcp.json` 与 skill。
食材数据库和 Clarabel 求解器都编译进二进制，**不需要 Python、运行时或本仓库**，
仓库移动或删除后插件照常工作。

`just release-plugin` 用 musl 目标构建**全静态** Linux 二进制，并把插件打成
`dist/food-solver-musl.tar.gz`；tarball 里带 `install.sh` / `uninstall.sh`，
目标机器解包后直接 `./food-solver/install.sh` 即可（复制文件、合并
`~/.agents/plugins/marketplace.json`、执行 `codex plugin add`，需要 `python3`
或 `jq`）。

二进制是**平台相关**的（当前为 darwin-arm64）：要在哪台机器上跑，就在哪台
机器上打包。本项目不做交叉编译；Linux 上直接 `cargo build --release` 即可
（需要静态链接时用 `--target x86_64-unknown-linux-musl`，走系统 `musl-gcc`）。
跨机分发与安装步骤见 `plugin/INSTALL.md`。

`plugin/` 是插件模板，`scripts/make_plugin.py` 负责打包；要改插件内容请改
模板，而不是安装后的副本。

`just mcp` 可以在当前仓库以 stdio 方式跑同一个 MCP 服务，便于本地调试。
