object #62
  name: "The First Room"
  parent: #3
  location: #-1
  owner: #36
  readable: true
  override "aliases" = {};

  override "description" = "This is all there is right now.";

  override "entrances" = {};

  override "exits" = {};

  verb "disfunc" (this none this) owner: #2 flags: "rxd"
    "Copied from The Coat Closet (#11):disfunc by Haakon (#2) Mon May  8 10:41:04 1995 PDT";
    if ((cp = caller_perms()) == (who = args[1]) || $perm_utils:controls(cp, who) || caller == this)
      "need the first check since guests don't control themselves";
      if (who.home == this)
        move(who, $limbo);
        this:announce("You hear a quiet popping sound; ", who.name, " has disconnected.");
      else
        pass(who);
      endif
    endif
  endverb

  verb "enterfunc" (this none this) owner: #2 flags: "rxd"
    "Copied from The Coat Closet (#11):enterfunc by Haakon (#2) Mon May  8 10:41:38 1995 PDT";
    who = args[1];
    if ($limbo:acceptable(who))
      move(who, $limbo);
    else
      pass(who);
    endif
  endverb

  verb "match" (this none this) owner: #36 flags: "rxd"
    "Copied from The Coat Closet (#11):match by Lambda (#50) Mon May  8 10:42:01 1995 PDT";
    m = pass(@args);
    if (m == $failed_match)
      "... it might be a player off in the body bag...";
      m = $string_utils:match_player(args[1]);
      if (valid(m) && !(m.location in {this, $limbo}))
        return $failed_match;
      endif
    endif
    return m;
  endverb

  verb "keep_clean" (this none this) owner: #2 flags: "rxd"
    "Copied from The Coat Closet (#11):keep_clean by Haakon (#2) Mon May  8 10:47:08 1995 PDT";
    if ($perm_utils:controls(caller_perms(), this))
      junk = {};
      while (1)
        for x in (junk)
          $command_utils:suspend_if_needed(0);
          if (x in this.contents)
            "This is old junk that's still around five minutes later.  Clean it up.";
            if (!valid(x.owner))
              move(x, $nothing);
              #2:tell(">**> Cleaned up orphan object `", x.name, "' (", x, "), owned by ", x.owner, ", to #-1.");
            elseif (!$object_utils:contains(x, x.owner))
              move(x, x.owner);
              x.owner:tell("You shouldn't leave junk in ", this.name, "; ", x.name, " (", x, ") has been moved to your inventory.");
              #2:tell(">**> Cleaned up `", x.name, "' (", x, "), owned by `", x.owner.name, "' (", x.owner, "), to ", x.owner, ".");
            endif
          endif
        endfor
        junk = {};
        for x in (this.contents)
          if (seconds_left() < 2 || ticks_left() < 1000)
            suspend(0);
          endif
          if (!is_player(x))
            junk = {@junk, x};
          endif
        endfor
        suspend(5 * 60);
      endwhile
    endif
  endverb

endobject
