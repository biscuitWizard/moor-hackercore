object #9
  name: "Version Management System Feature"
  parent: #74
  location: #83
  owner: #36
  readable: true

  override "aliases" = {"Version", "Management", "System", "Feature", "Version Management System Feature"};

  verb "@vms @vms/*" (any any any) owner: #36 flags: "rdx"
    if ($cu:switched_command(verb, "vms"))
      return;
    endif
    repo = worker_request("vms", {"repository_status"});
    output = {};
    output = {@output, tostr($su:left("Game", 15), ":  ", `repo["game"] ! E_RANGE => $server["core_history"][1] + " (Local)"')};
    output = {@output, tostr($su:right("Upstream", 15), ":  ", repo["upstream"])};
    output = {@output, ""};
    output = {@output, tostr($su:left("Last Change", 15), ":  ", "")};
    output = {@output, tostr($su:right("On", 15), ":  ", "")};
    output = {@output, tostr($su:right("Id", 15), ":  ", "")};
    output = {@output, ""};
    if (maphaskey(repo, "changes"))
    else
      output = {@output, "There are currently no changes."};
    endif
    player:tell_lines(output);
  endverb

endobject