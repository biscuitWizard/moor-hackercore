object #67
  name: "Display Options"
  parent: #68
  location: #-1
  owner: #36
  readable: true
  override "aliases" = {"Display Options"};

  override "names" = {"blank_tnt", "shortprep", "thisonly"};

  override "_namelist" = "!blank_tnt!shortprep!thisonly!";

  override "extras" = {};

  property "show_blank_tnt" (owner: #36, flags: "rc") = {"Treat `this none this' verbs like the others.", "Blank out the args on `this none this' verbs."};

  property "show_shortprep" (owner: #36, flags: "rc") = {"Display prepositions in full.", "Use short forms of prepositions."};

  property "show_thisonly" (owner: #36, flags: "rc") = {"./: will show ancestor properties/verbs if none on this.", "./: will not show ancestor properties/verbs."};

endobject
