contract C
{
    fn f() 
    {
        int x = --(,);
    }
}
// ----
// TypeError 9767: (59-64): Unary operator -- cannot be applied to type tuple(,)