object #3
  name: "Test Object"
  parent: #1
  location: #2
  owner: #2

  verb examine (this none this) owner: #2 flags: "rxd"
    player:tell("You look around.");
    return 1;
  endverb
endobject

