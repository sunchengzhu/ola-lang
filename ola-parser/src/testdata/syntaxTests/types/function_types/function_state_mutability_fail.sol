contract Test
{
  fn internalPureFunc(uint256 x) internal pure returns (uint256) { return x; }
  fn internalViewFunc(uint256 x) internal view returns (uint256) { return x; }
  fn internalMutableFunc(uint256 x) internal returns (uint256) { return x; }

  fn externalPureFunc(uint256 x) external pure returns (uint256) { return x; }
  fn externalViewFunc(uint256 x) external view returns (uint256) { return x; }
  fn externalPayableFunc(uint256 x) external payable returns (uint256) { return x; }
  fn externalMutableFunc(uint256 x) external returns (uint256) { return x; }

  fn funcTakesInternalPure(fn(uint256) internal pure returns(uint256) a) internal returns (uint256) { return a(4); }
  fn funcTakesInternalView(fn(uint256) internal view returns(uint256) a) internal returns (uint256) { return a(4); }
  fn funcTakesInternalMutable(fn(uint256) internal returns(uint256) a) internal returns (uint256) { return a(4); }

  fn funcTakesExternalPure(fn(uint256) external pure returns(uint256) a) internal returns (uint256) { return a(4); }
  fn funcTakesExternalView(fn(uint256) external view returns(uint256) a) internal returns (uint256) { return a(4); }
  fn funcTakesExternalPayable(fn(uint256) external payable returns(uint256) a) internal returns (uint256) { return a(4); }
  fn funcTakesExternalMutable(fn(uint256) external returns(uint256) a) internal returns (uint256) { return a(4); }

  fn tests() internal
  {
    funcTakesInternalPure(internalViewFunc); // view -> pure should fail
    funcTakesInternalPure(internalMutableFunc); // mutable -> pure should fail

    funcTakesInternalView(internalMutableFunc); // mutable -> view should fail

    funcTakesExternalPure(this.externalViewFunc); // view -> pure should fail
    funcTakesExternalPure(this.externalPayableFunc); // payable -> pure should fail
    funcTakesExternalPure(this.externalMutableFunc); // mutable -> pure should fail

    funcTakesExternalView(this.externalPayableFunc); // payable -> view should fail
    funcTakesExternalView(this.externalMutableFunc); // mutable -> view should fail

    funcTakesExternalPayable(this.externalPureFunc); // pure -> payable should fail
    funcTakesExternalPayable(this.externalViewFunc); // view -> payable should fail
    funcTakesExternalPayable(this.externalMutableFunc); // mutable -> payable should fail
  }
}
// ----
// TypeError 9553: (1580-1596): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) view returns (uint256) to fn (uint256) pure returns (uint256) requested.
// TypeError 9553: (1653-1672): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) returns (uint256) to fn (uint256) pure returns (uint256) requested.
// TypeError 9553: (1733-1752): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) returns (uint256) to fn (uint256) view returns (uint256) requested.
// TypeError 9553: (1813-1834): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) view external returns (uint256) to fn (uint256) pure external returns (uint256) requested.
// TypeError 9553: (1891-1915): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) payable external returns (uint256) to fn (uint256) pure external returns (uint256) requested.
// TypeError 9553: (1975-1999): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) external returns (uint256) to fn (uint256) pure external returns (uint256) requested.
// TypeError 9553: (2060-2084): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) payable external returns (uint256) to fn (uint256) view external returns (uint256) requested.
// TypeError 9553: (2144-2168): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) external returns (uint256) to fn (uint256) view external returns (uint256) requested.
// TypeError 9553: (2232-2253): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) pure external returns (uint256) to fn (uint256) payable external returns (uint256) requested.
// TypeError 9553: (2316-2337): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) view external returns (uint256) to fn (uint256) payable external returns (uint256) requested.
// TypeError 9553: (2400-2424): Invalid type for argument in fn call. Invalid implicit conversion from fn (uint256) external returns (uint256) to fn (uint256) payable external returns (uint256) requested.