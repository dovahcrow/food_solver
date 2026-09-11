# Food solver

食材质量和营养素质量的内部单位为 **g**，能量为 **J**；`KCAL = 4184`。

## 需求计算

`dog(age=3, weight=7, active=False, daily_kcal=None)` 当前只支持成年犬维持期。
年龄以年、体重以 kg 输入；不支持生长期、妊娠或哺乳期。一岁以上仍在
生长的大型犬也不应使用这个成年犬模型。

营养表统一采用 [FEDIAF 2025 Table III-3b（PDF 第 16 页）](https://europeanpetfood.org/wp-content/uploads/2025/09/FEDIAF-Nutritional-Guidelines_2025-ONLINE.pdf)
中每 1000 kcal 代谢能的成年犬推荐量，仅覆盖程序原本已有的营养素：

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

## 目标函数

每个参与优化的软目标为 `(营养供给 / target - 1)^2`。有上下限时 target
仍取中点，无上限时取下限的 1.05 倍。它是靠近目标值的偏好，不是区间外
才惩罚的模型。修复了线性项缺少 `/ target` 的问题。

启用 `minimize_usage` 时增加 `0.05 * (食材克数 / 食材上限)^2`，避免原来的
克数平方惩罚随批量大小放大。库存和需求一起乘 10 时，目标函数保持一致。

## 使用与验证

```sh
.venv/bin/python -m src opt -d 10
.venv/bin/python -m unittest discover -s tests -v
```

需要指定已确认的每日热量时可使用 `--daily-kcal`，例如命令形式为
`opt -d 10 --daily-kcal 500`；这个示例数值不是个体喂养建议。

每日需求指全天总量，尚未扣除商品粮贡献。当前命令中的 `foods_hard`
仍表示必须全部用完的批次重量，不会随 `day` 自动缩放。当前固定食材在
新的默认需求下可能无解；不要仅为得到解而提高热量。
