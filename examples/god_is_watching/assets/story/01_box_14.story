# God Is Watching -- Chapter I: Box 14
# What was recovered from the Von Lucis residence (1893-1894), read in 1903.

scene box_start:
  background von_lucis_study with fade
  music von_lucis
  "Item 1. From the notebook of Mary Von Lucis. November, 1893."
  "The letters from the Verlaine House come in pale envelopes that smell of perfume."
  "Father burns them in the study fireplace. I have counted three."
  show hugo tired at center with dissolve
  "The first one he laughed at. The second one he read twice."
  "The third one he left open on the desk while he went to fetch his coat."
  "I did not read it. His hands told me everything at supper."
  show hugo tired at right with dissolve
  show mary tired at left with dissolve
  hugo "Mary. You should be in bed."
  mary "So should you, Father."
  hugo "Go up. Lock your door tonight."
  mary "Why?"
  hugo "Because I asked."
  clear with dissolve
  jump box_the_letter

scene box_the_letter:
  background archive_office with dissolve
  music archive
  "The Archive. Under the notebook there is an envelope, pale, sealed again with somebody else's wax."
  "Whoever put it in the box wanted it kept, and did not want it read."

  choice:
    "Break the seal":
      set read_letter = true
      sound page_turn
      call give_item verlaine_letter
      "One word, and a signature. \"Von Lucis. Three days.\""
      "No \"Sir\". No \"Mister\". Only the name, as if it were a debt of its own."
      "You check the house register in the box. Three days after that letter, a Von Lucis debt is marked paid."
      call note debt_paid
    "Leave it sealed":
      "You put it back where it was. She never read it either."

  jump box_the_visit

scene box_the_visit:
  background von_lucis_bedroom with dissolve
  music von_lucis
  "Item 3. Mary's notebook again. The fourteenth of November, 1893."
  "I wake up and the candle has gone out on its own."
  "I go to the window first. I always look at the windows first."
  "Outside it is not night. Outside it is not anything."
  "There is a voice downstairs. Father's voice, talking and talking, filling a silence."
  "He talks about the bloodline. He talks about having no heir."
  "Then he says my name."
  "Nobody answers him. Nobody needs to."
  "I lie down again. I tell myself it was a dream, and for nine months it is."
  jump box_the_wind

scene box_the_wind:
  background von_lucis_bedroom with dissolve
  "Item 4. Testimony of Adelaide Roque, lady's maid. Taken in August, 1894."
  "July 31st. The wind begins at night. It is not a rain wind. It comes from inside the house."
  "August 1st. I go in before the six o'clock bell, as I have every day for nine years."
  show mary tired at left with dissolve
  show adelaide neutral at right with dissolve
  adelaide "Your hot water, miss. Miss? Are you unwell?"
  mary "Adelaide. Did I sleep last night?"
  adelaide "You did, miss. I put you to bed myself."
  "She pulls back the blanket."
  show mary afraid with dissolve
  show adelaide afraid with dissolve
  "Yesterday there was nothing. Today it is nine months."
  mary "Please. Don't tell anyone. Not yet."
  adelaide "Yes, miss."
  remove adelaide with dissolve
  "I told. I went straight to her father. May God forgive me. May God forgive all of us."
  clear with dissolve
  jump box_the_house

scene box_the_house:
  background von_lucis_hall with dissolve
  "Item 5. Mary's notebook. The last pages."
  "By the second day the whole house has chosen its word. Half say miracle. Half say sin."
  show hugo tired at right with dissolve
  show mary tired at left with dissolve
  mary "Father."
  "He looks at the floor, at the window, at the door. Anywhere but my face."
  "And that is when I understand. I am not being punished for something I did."
  "I am being punished for something he did."
  hugo "Go to your room, Mary."
  remove hugo with dissolve
  show mary resolved with dissolve
  "On the sixth of August the house scribe strikes my name from the register."
  "That night I take the servants' door, and one thing that is not mine."
  clear with dissolve
  jump box_the_register

scene box_the_register:
  background archive_office with dissolve
  music archive
  "The Archive. The Von Lucis register is in the box, and it has a column you have not seen in a register before."
  "Cause of the birth. For the eighth of August, 1894, it is empty."
  "The House expects the archivist who opens a box to complete it. In ink."

  choice final:
    "Write \"a miracle\"":
      set verdict = miracle
      "You write it. The pen scratches louder than it should."
    "Write \"a sin\"":
      set verdict = sin
      "You write it. It is what her father's house decided. Now it is what the House decided."
    "Write nothing, and initial the blank":
      set verdict = unknown
      add suspicion += 1
      "You initial the empty column. Somebody upstairs will ask why."

  jump box_santa_ilde

scene box_santa_ilde:
  background santa_ilde_door with dissolve
  music santa_ilde
  "Item 6. A note left at the Santa Ilde orphanage, folded four times. Copied by the House in 1894."
  "His name is Gabriel. I swear on my soul, which is not worth much anymore, I never knew the father."
  "An object goes with this note. It is not a keepsake and not an inheritance. It is proof."
  "One day he will ask where he came from. Give him this."
  jump box_the_river

scene box_the_river:
  background river_road with dissolve
  "Item 7. A report from the men her father sent. August 8th, 1894."
  "She was found on the old road by the river. She did not run."
  "She turned around and looked at us, as if she wanted to remember our faces."
  "She was searched, and the bank and the road as far as the bridge. What we were sent for was not found."
  jump box_the_question

scene box_the_question:
  background archive_office with dissolve
  music archive
  "Item 8. A loose sheet, in the hand of whoever compiled the box."
  "The debt was paid on the fifteenth of November. The child was born on the eighth of August."
  "Two hundred and sixty-six days. A whole pregnancy. An exact one."
  "Everyone repeats it with the word miracle, or with the word sin."
  "Nobody asks the only question that matters. Where was he for those nine months?"
  if verdict == miracle || verdict == sin:
    "You look at the word you wrote in the register, and it answers nothing."
  else:
    "You look at the blank you initialled. At least it isn't lying."
  jump notebook_start
