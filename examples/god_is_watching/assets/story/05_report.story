# God Is Watching -- Chapter V: The Report
# Back at the Archive. Three endings: report, silence, keeper.

scene report_start:
  background archive_office with fade
  music archive
  "Before you answer there is the box, and the question of what goes in with the report."
  call assemble
  show registrar stern at right with dissolve
  registrar "Report two hundred and twenty-two. Yours. What is in Santa Ilde?"
  if read_letter == true && saw_torn_page == true:
    call unlock thorough_reader

  if recognized_clara == true && trust >= 2:
    "You think of a wooden horse, a woman who waited nine years, and a man who kept a promise he couldn't remember making."
    choice final:
      "\"A boy with a blessing. The Von Lucis blood.\"":
        jump ending_report
      "\"Nothing the House needs.\"":
        jump ending_silence
      "\"Nothing.\" Then go back":
        jump ending_keeper
  else:
    choice final:
      "\"A boy with a blessing. The Von Lucis blood.\"":
        jump ending_report
      "\"Nothing the House needs.\"":
        jump ending_silence

scene ending_report:
  music ending
  set ending = report
  call unlock ending_report
  {player_name} "A boy with a blessing. The blood is Von Lucis. He doesn't know what he does."
  registrar "Thank you, {player_name}. That is what an archive is for."
  remove registrar with dissolve
  background santa_ilde_courtyard with dissolve
  "In December, three men in black knock at the door of Santa Ilde. Polite, in a way that makes you cold."
  "The administrator tells them there is no one new. It is true. The boy has been there for nine years."
  background archive_office with dissolve
  "Report 222 is filed under Von Lucis. Nobody asks you where he was for those nine months."
  "You never write another report that anyone reads twice."

scene ending_silence:
  music ending
  set ending = silence
  call unlock ending_silence
  {player_name} "Nothing the House needs. An old man, twenty-one children and a very quiet wing."
  if suspicion >= 2:
    registrar "You initialled a blank in the Von Lucis register. You wanted to go there before you'd read a page."
    registrar "Then we'll send someone who'll find something."
  else:
    registrar "Then Santa Ilde is closed. We have spent enough shadows on it."
  remove registrar with dissolve
  background archive_office with dissolve
  sound book_close
  "Box fourteen goes back on its shelf. You put the reports in after it, in order, and you don't write the last word of 221."

scene ending_keeper:
  music ending
  set ending = keeper
  call unlock ending_keeper
  {player_name} "Nothing."
  registrar "Nothing."
  {player_name} "An old man, twenty-one children and a very quiet wing. I'd like to be the one who keeps watching it."
  registrar "..."
  registrar "Report two hundred and twenty-two. Filed. Go."
  remove registrar with dissolve
  background santa_ilde_courtyard with dissolve
  show clara unveiled at left with dissolve
  show gabriel curious at right with dissolve
  "In the spring you take a room in the town below Santa Ilde. Every week you write the House a report."
  "Every week it says the same thing, and every week it is true: there's no family here. There's a boy."
  gabriel "Are you going to forget me?"
  {player_name} "No. That's my whole job now."
