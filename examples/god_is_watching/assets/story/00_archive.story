# God Is Watching -- Prologue: The Archive
# October 1903. The player is a new archivist of the House.

scene archive_start:
  background archive_office with fade
  music archive
  "October, 1903. The Archive of the House, two floors below the street."
  "The lamps are never put out down here. Paper is cheaper than daylight."
  show registrar neutral at right with dissolve
  registrar "You're the new one. Sit."
  registrar "I don't keep names I haven't needed yet. Write yours in the ledger."
  call ask_name player_name
  registrar "{player_name}. Good. Now forget it. Nobody down here will use it."
  {player_name} "What am I to work on, Registrar?"
  registrar "Two things that were never meant to be read together."
  call give_item box_14
  registrar "Box fourteen. What was recovered from the Von Lucis residence. Never catalogued."
  call give_item field_report 19
  registrar "And nineteen reports from our field post at Santa Ilde. Numbers two hundred and three to two hundred and twenty-one."
  show registrar stern with dissolve
  registrar "Report two hundred and twenty-one ends in the middle of a word."
  registrar "The man who wrote it has not come back. I want to know what he nearly wrote."

  choice:
    "Read everything first":
      set approach = careful
      {player_name} "I'll read the box first. All of it."
      registrar "Slow. Good. Slow people last longer here."
    "Ask to be sent to Santa Ilde":
      set approach = bold
      add suspicion += 1
      {player_name} "Why not send me to Santa Ilde and ask them?"
      registrar "Because the last three we sent forgot why they went."
      registrar "Read first. Then we'll see how eager you are."

  registrar "One more thing. What you read down here stays down here."
  registrar "And if a page ever feels like it is \"reading you back\", close the box and come and find me."
  remove registrar with dissolve
  sound latch
  "The box smells of smoke and lavender. The first item is a notebook, in a young woman's hand."
  jump box_start
