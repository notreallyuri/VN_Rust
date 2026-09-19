# God Is Watching -- Chapter II: The Notebook
# J. Moriarty's notebook (Santa Ilde, 1894) and the 1901 conversation, from Francis's copy.

scene notebook_start:
  background archive_office
  music archive
  "Report two hundred and fourteen has an annex: a copy of the administrator's notebook, made by our man Francis in one night."
  call give_item notebook_copy
  background santa_ilde_office
  music santa_ilde
  "July 12th, 1894. Santa Ilde. Lately I have begun to forget things."
  "It's all been signed since May. I leave Santa Ilde on my birthday and go to live with my sister."
  "So I am keeping a diary. I know that, eventually, I will forget to keep it too."
  "The children have behaved better than usual, ever since the arrival of the"
  "..."
  "Ever since the guest arrived."
  jump notebook_the_guest

scene notebook_the_guest:
  "July 14th. I am writing down what the guest looks like, because tomorrow I may not."
  show guest neutral at center
  "He is a tall man in a dark frock coat, thin enough to pity. He speaks little. He eats less."
  "..."
  "I read that back and it isn't right. He is broad in the shoulders. The coat is grey."
  "He eats normally, at the sisters' table."
  remove guest
  "July 23rd. The guest left two days ago. Nobody can tell me how long he stayed."
  show clara neutral at right
  show moriarty neutral at left
  moriarty "Sister Clara, the guest. How long was he with us?"
  clara "What guest, Mr. Moriarty?"
  moriarty "The man in the coat. At your table."
  clara "No guest stayed here this year."
  clear
  "Sister Clara served him dinner every night."
  jump notebook_the_lady

scene notebook_the_lady:
  background santa_ilde_door
  sound door_open
  "August 9th. A young woman came to the door after midnight, soaked through."
  show mary resolved at left
  show moriarty neutral at right
  "She did not ask for discretion. She did not offer money. She did not cry."
  moriarty "What is his name?"
  mary "Gabriel."
  moriarty "And his family name?"
  "She looked at the door."
  mary "Don't let anyone come looking for me."
  clear
  "I decided to take him in."
  jump notebook_the_men

scene notebook_the_men:
  background santa_ilde_office
  "August 11th. Men came this morning. Three of them, in black, polite in a way that makes you cold."
  show man_in_black neutral at right
  show moriarty neutral at left
  man_in_black "Good morning. Has a new child come in this week?"
  moriarty "No. No one new."
  man_in_black "Thank you for your time."
  clear
  "I lied to an armed man before breakfast, and I do not recognise myself."
  "Afterwards I unwrapped the boy's cloths to change him, and there was"
  "..."
  "(The sentence ends mid-line. The next page has been torn out at the stitching.)"
  jump notebook_the_page

scene notebook_the_page:
  background archive_office
  music archive
  "The Archive. Francis copied the notebook faithfully, down to the torn page. He copied the indentations too."
  "Whoever wrote on the missing page pressed hard enough to mark the next one."

  choice:
    "Hold the page to the lamp":
      set saw_torn_page = true
      "You tilt it against the lamp until the grooves catch the light."
      "\"...a page from a register, folded small. Folio 41. His name is on it, and a price...\""
      call note folio_41
    "Leave it":
      sound page_turn
      "You turn the page. Some things are torn out for a reason."

  jump notebook_1901

scene notebook_1901:
  background santa_ilde_office
  music santa_ilde
  "Part II of the annex. A conversation in the administrator's office, 1901, written down by Francis."
  show moriarty neutral at left
  show francis neutral at right
  moriarty "Read it. Then you'll understand why I talk about that half-year the way I do."
  francis "Then why did you stay? After everything in there?"
  moriarty "I grew attached. To my colleagues. To Gabriel."
  moriarty "Look after a child for seven years and then tell me you can pack your bags."
  francis "The torn page. What was in the boy's cloths?"
  moriarty "Nothing. Cloths."
  "The only quick answer of the night. It had been ready for years."
  moriarty "And Francis. Whoever it is you write to at night, tell them this."
  moriarty "There's no family here. There's a boy."
  clear
  background archive_office
  music archive
  "In Francis's copy, somebody at the field post wrote in the margin: \"A diversion. Not to be treated as information.\""

  choice:
    "Underline his last words":
      set believed_moriarty = true
      "You underline them twice, under the margin note, where the next reader will see both."
    "Let the margin note stand":
      "The field post knew the man. You didn't. You let it stand."

  jump reports_start
