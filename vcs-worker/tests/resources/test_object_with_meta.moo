object #9999
  name: "Test Object With Properties and Verbs"
  parent: #1
  location: #2
  owner: #2

  property test_property (owner: #2, flags: "rc") = "test value";
  property another_property (owner: #2, flags: "rc") = 123;

  verb test_verb (this none this) owner: #2 flags: "rxd"
    x = 42;
    return x;
  endverb

  verb another_verb (this none this) owner: #2 flags: "r"
    return "hello";
  endverb
endobject

