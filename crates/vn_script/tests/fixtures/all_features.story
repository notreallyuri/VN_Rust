# Covers every construct in SCRIPT.md. Golden input for lexer/parser/compiler tests.
# Keep it valid against the spec, even where the implementation lags behind.

scene start:
  # Character presentation (SCRIPT.md 2)
  background study_night
  show gabriel serious at left
  show mary neutral
  show gabriel worried

  # Dialogue and narration (SCRIPT.md 3)
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
  show gabriel worried
  "Things just got darker."
