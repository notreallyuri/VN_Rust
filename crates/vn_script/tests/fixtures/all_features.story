# Covers every construct in SCRIPT.md. Golden input for lexer/parser/compiler tests.
# Keep it valid against the spec, even where the implementation lags behind.

scene start:
  # Character presentation (SCRIPT.md 2)
  background study_night
  music night_theme
  sound door_knock
  show gabriel serious at left
  show mary neutral
  show gabriel worried
  show mary happy with dissolve
  remove mary with slide_left 0.8

  # Dialogue and narration (SCRIPT.md 3)
  voice gabriel_format
  gabriel "This is a really interesting format."
  mary "Is the parser working?"
  "The room goes quiet."
  mary "The sign says \"Closed\"."

  # Variables (SCRIPT.md 8)
  set met_mary = true
  set route = good
  set player_name = "Yuri"
  add affection += 1
  add affection -= 2

  # Conditional branching (SCRIPT.md 8.4)
  if affection >= 3 && met_mary == true:
    mary "You remembered."
  else:
    mary "You forgot."

  if route == good || affection < 0:
    "Something shifts."

  if player_name != "":
    # Interpolation (SCRIPT.md 3.5)
    mary "Nice to meet you, {player_name}."
    {player_name} "Nice to meet you, Mary."

  # Engine commands (SCRIPT.md 9)
  call give_item stick 1
  call unlock_route good
  call ask_name player_name

  # Rollback barriers (SCRIPT.md 10)
  choice final:
    "Keep the letter":
      "You fold it into your coat."
    "Burn the letter":
      "It curls into ash."
  commit

  # Choices (SCRIPT.md 4)
  choice:
    "Agree":
      "You nod silently."
      jump second_scene

    "Disagree":
      remove mary
      "You shake your head."
      jump second_scene


scene second_scene:
  clear
  background none
  music none
  show gabriel worried
  "Things just got darker."

  # Conditions and pictures on an option (SCRIPT.md 4.4, 4.5)
  choice:
    "Open the letter" when met_mary == true "You have not met her yet" image letter:
      "You break the seal."
    "Read the margin" unless route == good preview margin_note:
      "The handwriting is not hers."
    "Put it down":
      "You leave it where it was."
      jump nvl_scene


# Full-screen text pages (SCRIPT.md 1.2)
scene nvl_scene nvl:
  "The corridor narrows."
  "Her voice does not echo."
