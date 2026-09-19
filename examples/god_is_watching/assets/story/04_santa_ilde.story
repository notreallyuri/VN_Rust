# God Is Watching -- Chapter IV: Santa Ilde, 1903
# The archivist visits the orphanage in person.

scene ilde_start:
  background santa_ilde_courtyard
  music santa_ilde
  "November, 1903. Santa Ilde is smaller than the reports made it sound. Twenty-one windows. Somebody is singing."
  show moriarty older at right
  sound footsteps
  "The administrator comes out to meet you before you knock. He has worn the same coat for nine years."
  moriarty "You're not the bishopric. The bishopric sends letters."
  {player_name} "My name is {player_name}."

  choice:
    "Say the House sent you":
      add trust += 1
      {player_name} "I'm from the House. The Archive."
      moriarty "Honest. That's rarer than you'd think. Come in."
    "Say you're from the bishopric":
      add trust -= 1
      {player_name} "The bishopric sent me, after all."
      moriarty "The bishopric doesn't know how many children live here. Come in anyway."

  jump ilde_office

scene ilde_office:
  background santa_ilde_office
  show moriarty older at left
  moriarty "Ask your questions. I'll tell you which ones I've forgotten the answers to."
  if saw_torn_page == true:
    "The folio number is on the tip of your tongue."
    choice:
      "Ask about folio 41":
        show moriarty wary
        {player_name} "What was in the boy's cloths, Mr. Moriarty? Folio 41?"
        moriarty "Who else has read that page?"
        if believed_moriarty == true:
          add trust += 1
          {player_name} "Only me. And I underlined what you said to Francis."
          moriarty "Then you already know what I'd say. A boy. Not a family."
        else:
          add trust -= 1
          {player_name} "The field post."
          moriarty "Then you have your answer, and so do they."
      "Keep it to yourself":
        "You keep the number to yourself. He watches you do it."
  else:
    moriarty "No questions? The last ones had a list."
  jump ilde_nursery

scene ilde_nursery:
  background nursery
  show gabriel neutral at center
  "The nursery is empty except for one boy, nine years old, sitting on the floor by the window."
  "Beside him there is a painted wooden horse. It was not there when you came in."
  show gabriel curious
  gabriel "Are you going to forget me too? The last man forgot me."

  choice:
    "Ask him where the horse came from":
      add trust += 1
      {player_name} "Where did the horse come from?"
      gabriel "I wanted it."
      "He holds it out to you. Moriarty, in the doorway, lets out a breath."
      call give_item wooden_horse
    "Write it down":
      add suspicion -= 1
      "You note the time, the toy, the window. The House will want to know. The boy watches the pencil."

  remove gabriel
  jump ilde_hot_water

scene ilde_hot_water:
  background santa_ilde_office
  show clara guarded at right
  "The sister who brings you tea keeps her eyes down. She has been at Santa Ilde longer than anyone but the administrator."
  clara "Your hot water, miss."
  if read_letter == true || saw_torn_page == true:
    "Four words. You have read them before, in a testimony taken in August 1894."
    choice:
      "Call her Adelaide":
        set recognized_clara = true
        call note hot_water
        show clara unveiled
        {player_name} "Thank you, Adelaide."
        "The cup does not shake. She has been waiting nine years for somebody to say it."
        clara "I told her father. I went straight to him, like a good servant."
        clara "I've spent every day since deciding things for her son instead."
        clara "The floorboard was mine, too. Your people were three steps behind. I made sure of it."
      "Thank her, and let it go":
        "You thank her. She leaves. Some names are safer where they are."
  else:
    "You thank her. She leaves without a sound."
    sound door_close

  remove clara
  jump ilde_leaving

scene ilde_leaving:
  background santa_ilde_door
  show moriarty older at center
  moriarty "You'll write something about us. Everyone who comes here does."
  if trust >= 2:
    moriarty "Write it carefully. He's the only reason this house still has children in it."
  else:
    moriarty "Write what you like. We've been written about before."
  remove moriarty
  "You sign the visitors' book on your way out. Whatever you do next, you have been to Santa Ilde."
  commit
  jump report_start
