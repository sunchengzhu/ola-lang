contract b {
    struct c {
        u256 [2 ** 253] a;
    }

    c d;

    fn e()  {
        c storage x = d;
        x.a[0];
        fn()[3**44] storage fs;
    }
}
// ----
// Warning 7325: (66-67): Type struct b.c covers a large part of storage and thus makes collisions likely. Either use mappings or dynamic arrays and allow their size to be increased only in small quantities per transaction.
// Warning 7325: (66-67): Type u256[14474011154664524427946373126085988481658748083205070504932198000989141204992] covers a large part of storage and thus makes collisions likely. Either use mappings or dynamic arrays and allow their size to be increased only in small quantities per transaction.
// Warning 7325: (111-112): Type struct b.c covers a large part of storage and thus makes collisions likely. Either use mappings or dynamic arrays and allow their size to be increased only in small quantities per transaction.
// Warning 7325: (111-112): Type u256[14474011154664524427946373126085988481658748083205070504932198000989141204992] covers a large part of storage and thus makes collisions likely. Either use mappings or dynamic arrays and allow their size to be increased only in small quantities per transaction.
// Warning 7325: (152-169): Type fn ()[984770902183611232881] covers a large part of storage and thus makes collisions likely. Either use mappings or dynamic arrays and allow their size to be increased only in small quantities per transaction.
// Warning 2072: (152-180): Unused local variable.
