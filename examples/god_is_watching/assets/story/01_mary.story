# God Is Watching -- Part 1: The Von Lucis House
# POV: Mary Von Lucis
# Source: "Box 14 -- Material recovered from the Von Lucis residence" (items 1-8)

scene mary_start:
  "November, 1893. The Von Lucis residence."
  "The letters from the Verlaine House come in pale envelopes that smell of perfume."
  "Father burns them in the study fireplace. I have counted three."

  show hugo tired
  "The first one he laughed at. The second one he read twice."
  "The third one he left open on the desk while he went to fetch his coat."

  choice:
    "Read it":
      add curiosity += 1
      call give_item verlaine_letter 1
      "One word, and a signature. Von Lucis. Three days."
      "No 'Sir'. No 'Mister'. Only the name, as if it were a debt of its own."
    "Leave it alone":
      add obedience += 1
      "I do not read it. I do not need to. His hands told me everything at supper."

  hugo "Mary. You should be in bed."
  mary "So should you, Father."
  hugo "Go up. Lock your door tonight."
  mary "Why?"
  hugo "Because I asked."
  remove hugo
  jump mary_the_visit

scene mary_the_visit:
  # Item 3 -- Hugo's notebook, 14 November 1893, seen from the other side of the door
  "The fourteenth of November. I wake up and the candle has gone out on its own."
  "I go to the window first. I always look at the windows first."
  "Outside it is not night. Outside it is not anything."
  "There is a voice downstairs. Father's voice, talking and talking, filling a silence."
  "He talks about the bloodline. He talks about having no heir."
  "He says the blessing is in his daughter, and that she is too weak to carry it."
  "Then he says my name."
  "Nobody answers him. Nobody needs to."
  "I lie down again. I tell myself it was a dream, and for nine months it is."
  jump mary_breakfast

scene mary_breakfast:
  # Item 3 -- 16 November 1893
  "Two days later, the three candles in the hallway light themselves as I pass."
  "Father is at the table already. He is smiling. His right hand is not trembling."
  show hugo neutral

  choice:
    "Ask about the candles":
      add curiosity += 1
      mary "The hallway candles were lit when I came down."
      hugo "Were they? One of the maids, then."
      "None of the maids are awake before six."
    "Ask about the letters":
      add suspicion += 1
      mary "No more letters from Verlaine?"
      hugo "It has been settled."
      mary "Settled by whom?"
      "He does not answer, and I realise he never asked."

  mary "Did you sleep well, Father?"
  hugo "Yes."
  "He does not look up from his plate when he says it."
  clear
  jump mary_the_wind

scene mary_the_wind:
  # Item 4 -- Testimony of Adelaide Roque
  "July 31st, 1894. The wind begins at night."
  "It is not a rain wind. It comes from inside the house."
  "It knocks on the doors from the wrong side."
  "I do not remember lying down."
  "August 1st. Adelaide comes in before the six o'clock bell, as she has every day for nine years."
  show adelaide neutral
  adelaide "Your hot water, miss. Miss? Are you unwell?"
  mary "Adelaide. Did I sleep last night?"
  adelaide "You did, miss. I put you to bed myself."
  mary "I don't remember."
  "I pull back the blanket."
  show adelaide afraid
  "Yesterday there was nothing. Today it is nine months."
  "I do not scream. I look at it the way you look at a letter delivered to the wrong address."

  choice:
    "Ask her not to tell anyone":
      mary "Please. Don't tell anyone. Not yet."
      adelaide "Miss, I..."
      mary "Please."
      adelaide "Yes, miss."
    "Say nothing":
      "I say nothing. She has dressed me since I was thirteen. She will know what to do."
      "She does. She goes straight to my father."

  remove adelaide
  "She tells. I do not blame her. May God forgive her. May God forgive all of us."
  jump mary_the_house

scene mary_the_house:
  # Items 1 and 5 -- the register and the internal order
  "By the second day the whole house has chosen its word. Half say miracle. Half say sin."
  "Father will not look at me."
  show hugo tired
  mary "Father."
  "He looks at the floor, at the window, at the door. Anywhere but my face."
  "And that is when I understand."
  "I spent nine months, in one night, thinking I was being punished for something I did."
  "I am being punished for something he did."
  hugo "Go to your room, Mary."
  remove hugo
  "On the sixth of August, the house scribe strikes my name from the register."
  "That same evening, a sealed order goes down to the guard."
  "I never read it. I do not need to. I hear the word they use for it on the stairs."
  "Purification."
  "That night I take the servants' door, and I take one thing with me that is not mine."
  jump mary_santa_ilde

scene mary_santa_ilde:
  # Item 6 -- the note left at the Santa Ilde orphanage
  "The wind stops on its own, the way it started. By then he is in my arms."
  "Santa Ilde is a stone house at the edge of the province. It is past midnight. I am soaked."
  show moriarty neutral
  moriarty "Madam, it's very late. What can I do for you?"
  mary "Keep him. Here."
  moriarty "What is his name?"
  mary "Gabriel."
  moriarty "And his family name?"
  "I look at the door."
  "The rest of his name is the part they are coming to kill."
  mary "Please. Don't let anyone come looking for me."
  moriarty "Madam, if you are in some kind of trouble..."
  mary "Don't let anyone come looking for me. It won't help. You'll only get hurt."
  remove moriarty

  "Before I go, I leave a note with one of the sisters, folded four times."
  "His name is Gabriel. I swear on my soul, which is not worth much anymore, I never knew the father."
  "An object goes with this note. It is not a keepsake and not an inheritance. It is proof."
  "One day he will ask where he came from. Give him this."
  jump mary_the_river

scene mary_the_river:
  # Item 7 -- execution report
  "August 8th. The old road, and the river beside it."
  "I hear them long before I see them. They do not bother to be quiet."

  choice:
    "Run":
      "I run until the road bends toward the bridge."
      "Then my legs stop on their own, as if someone else decided it for them."
    "Keep walking":
      "I keep walking. There is nothing left to run to."

  "I stop. I turn around and face them."
  "It is not courage. I only want to see the faces of the men my father sent."
  "They search me. They search the bank and the road as far as the bridge."
  "They do not find what they came for."
  "Good."
  jump mary_box_14

scene mary_box_14:
  # Item 8 -- the compiler's loose sheet
  clear
  "Box 14. Recovered from the Von Lucis residence. Not catalogued."
  "The debt was paid on the fifteenth of November. The child was born on the eighth of August."
  "Two hundred and sixty-six days. A whole pregnancy. An exact one."
  "Everyone repeats it with the word miracle, or with the word sin."
  "Nobody asks the only question that matters."
  "Where was he for those nine months?"
  "Not in her. Something carried that child somewhere else for nine months."
  "And at the end, it put him back into the right womb, in the right house, on the night the wind"
  jump moriarty_start
