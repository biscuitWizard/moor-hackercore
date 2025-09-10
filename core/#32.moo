object #32
  name: "Mr. Spell"
  parent: #1
  location: #-1
  owner: #36
  readable: true
  override "aliases" = {"Mr. Spell", "spell"};

  override "description" = "For help on using the speller, use 'help spelling' and 'help @spell'.";

  verb "valid" (this none this) owner: #36 flags: "rxd"
    return call_function("spellcheck", args[1]) == 1 || args[1] in player.dict;
  endverb

  verb "get_input" (this none this) owner: #2 flags: "rxd"
    set_task_perms(caller_perms());
    source = args[1];
    data = {};
    ref = $code_utils:parse_propref(source);
    if (ref)
      "User entered a prop. Deal with it.";
      {thing, prop} = ref;
      thing = $string_utils:match_object(thing, player.location);
      if (!valid(thing))
        player:tell("No such object: ", ref[1]);
        data = $failed_match;
      elseif (!prop || `thing.(tostr(prop)) ! ANY' == E_PROPNF)
        player:tell("There is no such property `", prop, "' on object ", thing, ".");
        data = $failed_match;
      else
        data = `thing.(tostr(prop)) ! ANY';
        if (typeof(data) == STR)
          data = {data};
        endif
        $command_utils:suspend_if_needed(3);
        if (typeof(data) == ERR)
          player:tell("Error: ", tostr(data));
          data = $failed_match;
        elseif (typeof(data) != LIST)
          player:tell("Spellchecker needs a string or list as input.");
          data = $failed_match;
        endif
      endif
    else
      ref = $code_utils:parse_verbref(source);
      if (ref)
        "User entered a verb. Deal with it.";
        {thing, verb} = ref;
        thing = $string_utils:match_object(thing, player.location);
        if (!valid(thing))
          player:tell("No such object: ", ref[1]);
          data = $failed_match;
        elseif (`verb_info(thing, verb) ! ANY' == E_VERBNF)
          player:tell("There is no such verb `", verb, "' on object ", thing, ".");
          data = $failed_match;
        else
          data = `verb_code(thing, verb) ! ANY => {}';
          for i in [1..length(data)]
            if (!index(data[i], "\""))
              data[i] = "";
            else
              data[i] = data[i][index(data[i], "\"") + 1..$];
              data[i] = data[i][1..rindex(data[i], "\"") - 1];
              foo = "";
              while (index(data[i], "\""))
                foo = foo + data[i][1..index(data[i], "\"") - 1];
                foo = foo + " ";
                data[i] = data[i][index(data[i], "\"") + 1..$];
                data[i] = data[i][index(data[i], "\"") + 1..$];
              endwhile
              if (foo == "")
                foo = data[i];
              else
                foo = foo + data[i];
              endif
              data[i] = $string_utils:trim(foo);
            endif
          endfor
        endif
      else
        "User entered word/phrase on command line.";
        data = {argstr};
      endif
    endif
    for i in [1..length(data)]
      $command_utils:suspend_if_needed(1);
      if (typeof(data[i]) != STR)
        data[i] = "";
      endif
      data[i] = $string_utils:strip_chars(data[i], "!@#$%^&*()_+1234567890={}[]`<>?:;,./|\"~'");
    endfor
    return data;
  endverb

  verb "guess_words" (this none this) owner: #36 flags: "rxd"
    {nastyword} = args;
    guesses = call_function("spellcheck", nastyword);
    "Transpose adjacent characters";
    nastyword = nastyword + " ";
    for i in [1..length(nastyword) - 1]
      foo = nastyword[1..i - 1] + nastyword[i + 1] + nastyword[i] + nastyword[i + 2..$];
      foo = $string_utils:trim(foo);
      if (this:valid(foo))
        guesses = setadd(guesses, foo);
      endif
      if (ticks_left() < 500 || seconds_left() < 2)
        suspend(0);
      endif
    endfor
    nastyword = $string_utils:trim(nastyword);
    "Erase each character - check for an extra typoed character";
    for i in [1..length(nastyword)]
      foo = nastyword[1..i - 1] + nastyword[i + 1..$];
      if (this:valid(foo))
        guesses = setadd(guesses, foo);
      endif
      if (ticks_left() < 500 || seconds_left() < 2)
        suspend(0);
      endif
    endfor
    "Alter one character";
    for i in [1..length(nastyword)]
      for ii in ({"a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "'", "-"})
        foo = nastyword[1..i - 1] + ii + nastyword[i + 1..$];
        if (this:valid(foo))
          guesses = setadd(guesses, foo);
        endif
      endfor
      if (ticks_left() < 500 || seconds_left() < 2)
        suspend(0);
      endif
    endfor
    "insert one character";
    for i in [1..length(nastyword)]
      for ii in ({"a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "'", "-"})
        foo = nastyword[1..i - 1] + ii + nastyword[i..$];
        if (this:valid(foo))
          guesses = setadd(guesses, foo);
        endif
      endfor
      if (ticks_left() < 500 || seconds_left() < 2)
        suspend(0);
      endif
    endfor
    "Clean up and go home";
    guesses = $list_utils:sort(guesses);
    return guesses;
  endverb

  verb "help_msg" (none none none) owner: #36 flags: "rxd"
    return this.description;
  endverb

  verb "proxy_for_core" (this none this) owner: #2 flags: "rxd"
    return this;
  endverb

endobject
