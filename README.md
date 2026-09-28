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
直接使用 `RecipeSolver` 时需显式调用：

```python
solver.add_nutrient_ratio(Nutrient.CALCIUM, Nutrient.PHOSPHORUS, 1., 2.)
```

比例不随批次天数缩放。此接口添加 `(P-Ca) @ x <= 0` 和
`(Ca-2*P) @ x <= 0` 两行线性约束（这里 Ca、P 是食材营养密度行，
不是二次目标矩阵）。磷的正下限保证分母非零。

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

当需求只有正下限、上限为 `None` 时，`RecipeSolver` 默认增加一个隐式软上限：
`U = 1.5 * L`。超过 U 后按 `max(0, 供给/U - 1)^2` 惩罚，包括蛋白质这类
硬下限需求；它只影响可行方案之间的选择，不会造成无解。倍数可配置：

```python
RecipeSolver(implicit_soft_upper_multiplier=1.2)   # 更接近最低量
RecipeSolver(implicit_soft_upper_multiplier=2.0)   # 更宽松
RecipeSolver(implicit_soft_upper_multiplier=None)  # 关闭隐式软上限
```

显式上限始终优先。最低量为 0 时不会自动生成上限。这个倍数是配方偏好，
不是 FEDIAF 的安全上限，也不能据此判断某营养素超过后是否有毒性。

营养报告显示实际参与优化的区间，包括隐式软上限，并注明其倍数。颜色为：
红色表示低于最低值，黄色表示高于显式或隐式上限，绿色表示位于区间内；
NOT_REQUIRED 项使用较弱的灰色/黄色提示。黄色超量不等于毒性或硬约束失败。
`--detail true` 的食材分项、营养总量和范围都按每天显示，即使求解的是多日批次。
实现使用 CVXPY 的 `cp.Variable`、`H @ x`、`cp.square(cp.pos(...))`
直接表达这些公式。CVXPY 自动生成辅助变量并交给 CLARABEL 求解，
无需手工拼接 P/q/G/h；`x.value` 只包含食材克数。
`recipe.py` 中保留了标准二次规划的推导注释。

依赖为 CVXPY 1.9.2+，具体版本由 `uv.lock` 固定，不再依赖 CVXOPT。
`solver.problem.status` 和 `.value` 提供原生结果；保留 `sol['x']`、
`sol['status']`、`sol['primal objective']` 兼容接口。
只有 `OPTIMAL` 返回成功，不可行时返回 False；无成功结果时 `amount()`
报错，防止取到旧配方。底层求解器异常会正常抛出。

官方文档：[CVXPY 建模规则](https://www.cvxpy.org/tutorial/dcp/index.html)、
[求解器接口](https://www.cvxpy.org/tutorial/solvers/index.html)。

启用 `minimize_usage` 时增加 `0.05 * (食材克数 / 食材上限)^2`，避免原来的
克数平方惩罚随批量大小放大。库存和需求一起乘 10 时，目标函数保持一致。

## 使用与验证

```sh
uv sync
.venv/bin/python -m src opt -d 10
.venv/bin/python -m unittest discover -s tests -v
```

需要指定已确认的每日热量时可使用 `--daily-kcal`，例如命令形式为
`opt -d 10 --daily-kcal 500`；这个示例数值不是个体喂养建议。

每日需求指全天总量，尚未扣除商品粮贡献。当前命令中的 `foods_hard`
仍表示必须全部用完的批次重量，不会随 `day` 自动缩放。当前固定食材在
新的默认需求下可能无解；不要仅为得到解而提高热量。

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

旧 JSON 缓存不会自动多出以前被丢弃的氨基酸/脂肪酸。可以显式重新获取（Rust 或 Python 前端均可，二者产出的缓存逐字节一致）：

```sh
just refresh-foods RICE EGG CHICKEN_BREAST   # Rust 抓取器（food fetch）
just refresh-foods                           # 全部可抓取食材
just refresh-foods --list                    # 列出食材、可用来源与当前选择
just refresh-foods-py RICE EGG               # Python 参考实现
```

这个命令会联网并覆盖指定食材缓存；来源本身未提供的字段仍会缺失。
不要把其他食材的数据或总蛋白估算值填作已测量的氨基酸含量。
报告中的 `incomplete data: ...` 列出实际使用但该营养素数据不全的食材。
优化仍使用已知贡献（未知项按零贡献处理），组合项只有一项已知时使用
部分和并标记不完整；不再通过读取 defaultdict 把缺失字段写成“已知零”。

## 代码结构：Rust 库 + CLI + MCP

求解器现在是 Rust 工作区，规划逻辑在一个库里，CLI 和 MCP 是它的两个前端：

- `rust/food-core/`：`PlanRequest`（食材 + 天数 + 犬只档案）进，`PlanResult`
  （配方 + 每日营养报告）出。`IngredientSpec` 用上下界描述食材：相等即固定
  用量，`0 ~ 上限` 表示可选。数量都是**整批克数**，`days` 只缩放需求。
- `rust/food-cli/`：`food` 命令。
- `rust/food-mcp/`：`food-mcp`，把同一流程暴露成 `list_foods`、`get_needs`、
  `solve_recipe` 三个 MCP 工具。`solve_recipe` 与 `get_needs` 的 `weight`、
  `days`、`age` 是必填项（没有默认值），犬只档案未知时应先询问用户。
- `rust/food-cli/` 的 `fetch` 子命令（`food fetch`）：抓取 USDA portal、
  《中国食物成分表》、USDA SR Legacy 与日本 MEXT 四种来源，各自写进
  `foods/{FOOD}_{Source}.json`；`expand-sr-legacy` 子命令把 SR Legacy 的
  全量 JSON 展开成同样的逐食材文件。USDA/中国两条路径移植自 Python 的
  `food_getters/{usda,chinanutri}.py`，产出与 Python 逐字节相同。
- 缓存布局：同一种食材每个来源一个文件（如 `BAICAI_Chinanutri.json`、
  `BEEF_SrLegacy.json`），文件体只存营养素，来源由文件名承载。`build.rs`
  把**全部**来源都编进二进制，运行期由 `src/foods.rs` 的 `CHOOSE` 表决定用
  哪一份——CLI 和 MCP 传入的始终只是食材名，名字 + 来源在内部合成后再查表。
  换来源只改 `CHOOSE` 一行并重建，不必重新抓取。
- `rust/food-core/build.rs`：构建时读取 `foods/{FOOD}_{Source}.json`（外加 Python 源码里
  三条内联配方）生成内嵌食材营养表，`src/foods.rs` 再用 `include!` 引进来。

求解器用 [Clarabel](https://github.com/oxfordcontrol/Clarabel.rs)（牛津大学，
CVXPY 的默认求解器），不是 `cvxrust`（只有 0.1.0，不成熟）。目标函数与
Python 版完全一致：只有下限的软需求用 `max(0, (L - y) / L)^2`，有上下限的
用两侧相对平方损失，另加 `U = 1.5L` 的隐式软上限；硬约束与钙磷比不变。
Rust 版原本与 Python 输出数值一致；缓存改为按来源取行后，被 `CHOOSE`
切到 SR Legacy 的 18 个食材数值已变化，参考测试因此记录的是当前基线
（见 `rust/food-core/tests/reference.rs`）。

```sh
just build          # cargo build --release
just test           # cargo test --workspace
just run -d 10      # 运行 Rust CLI
just check          # cargo check
just clippy         # cargo clippy（-D warnings）
```

CLI 支持 `-d/--day`、`--detail`、`--daily-kcal`、`--weight`、`--age`、
`--active`，以及可重复的 `-i FOOD:GRAMS[:optional|minimize|fixed]`；
不写 `-i` 时沿用历史默认批次。`food --foods` 列出全部食材。

## Python 参考实现

`src/` 仍保留为 Python 参考实现，主要用来和 Rust 结果对照：`just opt -d 10`
运行它，`just refresh-foods-py` 走它的食材抓取（仅 USDA/中国两条来源）。
数据抓取已由 Rust 的 `food fetch` 覆盖（`just refresh-foods`），因此更新
食材数据不再依赖 Python。MCP 支持只保留在 Rust 的 `food-mcp`，Python 侧的
`mcp_server` 已不存在，对应的 Python 测试也已移除。
食材 JSON 更新后重新构建即可（`build.rs` 会自动重新生成内嵌表）。
SR Legacy 的氨基酸/脂肪酸面板比 portal 视图全，`CHOOSE` 里已把 18 个食材
切到 `SrLegacy`（因此这些食材的求解数值与旧 Python 结果不再一致）；MEXT
是唯一提供碘、生物素、泛酸的来源，抓取脚本已接好，`MEXT` 表填好编号即可用。

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
