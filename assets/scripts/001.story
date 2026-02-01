scene start:
  bg office
  music calm_theme

  show gabriel serious

  gabriel "This is a really interesting format."

  show mary serious

  mary "..."
  mary "Is the parser working?"

  choice:
    "Yes":
      show crownley serious
      "And the room goes silent"
      remove mary

      jump second_scene

    "No":
      clear
      # Ends here


scene second_scene:
  bg office_dark
  music dark_music

  show gabriel worried
  show crownley serious

  crownley "So this is the second scene"

  "'Indeed it is', thought Gabriel"

  gabriel "Indeed it is"

  "He actually spoke out loud"
  clear
