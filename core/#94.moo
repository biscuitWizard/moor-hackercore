object #94
  name: "Generic Gendered Object"
  parent: #1
  location: #-1
  owner: #2
  readable: true
  override "key" = 0;

  override "aliases" = {"Generic Gendered Object"};

  property "gender" (owner: #2, flags: "rc") = "neuter";

  property "pqc" (owner: #2, flags: "rc") = "its";

  property "pq" (owner: #2, flags: "rc") = "its";

  property "ppc" (owner: #2, flags: "rc") = "Its";

  property "pp" (owner: #2, flags: "rc") = "its";

  property "prc" (owner: #2, flags: "rc") = "Itself";

  property "pr" (owner: #2, flags: "rc") = "itself";

  property "poc" (owner: #2, flags: "rc") = "It";

  property "po" (owner: #2, flags: "rc") = "it";

  property "psc" (owner: #2, flags: "rc") = "It";

  property "ps" (owner: #2, flags: "rc") = "it";

  property "verb_subs" (owner: #2, flags: "rc") = {};

  verb "set_gender" (this none this) owner: #2 flags: "rxd"
    "set_gender(newgender) attempts to change this.gender to newgender";
    "  => E_PERM   if you don't own this or aren't its parent";
    "  => Other return values as from $gender_utils:set.";
    if (!($perm_utils:controls(caller_perms(), this) || this == caller))
      return E_PERM;
    else
      result = $gender_utils:set(this, args[1]);
      this.gender = typeof(result) == STR ? result | args[1];
      return result;
    endif
  endverb

  verb "verb_sub" (this none this) owner: #2 flags: "rxd"
    "Copied from generic player (#6):verb_sub by ur-Rog (#6349) Fri Jan 22 11:20:11 1999 PST";
    "This verb was copied by TheCat on 01/22/99, so that the generic gendered object will be able to do verb conjugation as well as pronoun substitution.";
    text = args[1];
    if (a = `$list_utils:assoc(text, this.verb_subs) ! ANY')
      return a[2];
    else
      return $gender_utils:get_conj(text, this);
    endif
  endverb

endobject
