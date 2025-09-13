object #84
  name: "Singleton Warehouse"
  parent: #8
  location: #62
  owner: #36
  readable: true
  override "key" = 0;

  override "aliases" = {"Singleton Warehouse", "warehouse"};

  override "dark" = 0;

  override "opened" = 1;

  verb "list" (any in this) owner: #36 flags: "rxd"
    if (this.contents)
      player:tell(".singleton objects:");
      player:tell("----------------------");
      first = 1;
      for thing in (this.contents)
        if (!first)
          player:tell();
        endif
        player:tell($string_utils:nn(thing), ":");
        `thing:look_self() ! ANY => player:tell("<<Error printing description>>")';
        first = 0;
      endfor
      player:tell("----------------------");
    else
      player:tell("No objects in ", this.name, ".");
    endif
  endverb

endobject
