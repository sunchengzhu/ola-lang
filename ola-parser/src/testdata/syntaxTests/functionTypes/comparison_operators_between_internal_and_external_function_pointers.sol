contract C {
    fn external_test_function() external {}
    fn internal_test_function() internal {}

    fn comparison_operator_between_internal_and_external_function_pointers() external returns (bool) {
        fn () external external_function_pointer_local = this.external_test_function;
        fn () internal internal_function_pointer_local = internal_test_function;

        assert(
            this.external_test_function == external_function_pointer_local &&
            internal_function_pointer_local == internal_test_function
        );
        assert(
            internal_function_pointer_local != external_function_pointer_local &&
            internal_test_function != this.external_test_function
        );

        return true;
    }
}
// ----
// TypeError 2271: (606-672): Operator != not compatible with types fn () and fn () external
// TypeError 2271: (688-741): Operator != not compatible with types fn () and fn () external