contract test {
    mapping(ufixed8x1 => string) fixedString;
    fn f() public {
        fixedString[0.5] = "Half";
    }
}
// ----
// UnimplementedFeatureError: Not yet implemented - FixedPointType.
