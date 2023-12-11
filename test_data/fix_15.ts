interface CSSConditionRule extends CSSGroupingRule {
    readonly conditionText: string;
}

declare var CSSConditionRule: {
    prototype: CSSConditionRule;
    new(): CSSConditionRule;
};
