# God Is Watching -- Chapter 2: Santa Ilde
# POV: J. Moriarty, administrator of the Santa Ilde orphanage
# Source: "Annex to Report No. 214" (Moriarty's notebook, 1894, and the 1901 conversation)

scene moriarty_start:
  "July 12th, 1894. Santa Ilde Orphanage."
  "Lately I have begun to forget things."
  "It's all been signed since May. I leave Santa Ilde on my birthday and go to live with my sister."
  "I spent a whole week planning September in this house before someone reminded me."
  "In September I will not be in this house."
  "So I am keeping a diary. I know that, eventually, I will forget to keep it too."
  "Nothing much to write today. The children behaved better than usual."
  "They have, ever since the arrival of the"
  "..."
  "Ever since the guest arrived."
  jump moriarty_the_guest

scene moriarty_the_guest:
  "July 14th. I am writing down what the guest looks like, because tomorrow I may not."
  show guest neutral
  "He is a tall man in a dark frock coat, thin enough to pity. He speaks little. He eats less."
  "..."
  "I read that back and it isn't right."
  "He is a man of ordinary build, broad in the shoulders. The coat is grey, not dark."
  "He eats normally, at the sisters' table."
  remove guest

  choice:
    "Cross out the first description":
      "I lift the pen and cannot decide which one to strike."
    "Cross out the second description":
      "I lift the pen and cannot decide which one to strike."

  "I leave both. Tomorrow I will look again and decide."
  jump moriarty_eighteenth

scene moriarty_eighteenth:
  "July 18th."
  "I understand the children being quiet for a day after the guest arrived."
  "But it has been a week."
  "Something is wrong."
  jump moriarty_eighteenth_again

scene moriarty_eighteenth_again:
  "July 18th."
  "I understand the children being quiet for a day after the guest arrived."
  "But it has been a week and a half."
  "Something is wrong."
  jump moriarty_departure

scene moriarty_departure:
  "July 23rd. The guest left two days ago."
  "Little by little the children are back to normal, and so is my workload."
  "Nobody can tell me how long he stayed. I asked three people."
  show clara neutral
  moriarty "Sister Clara, the guest. How long was he with us?"
  clara "What guest, Mr. Moriarty?"
  moriarty "The man in the coat. At your table."
  clara "No guest stayed here this year."
  remove clara
  "One said a week. One said a month. Sister Clara served him dinner every night."
  jump moriarty_the_wind

scene moriarty_the_wind:
  "August 4th. The wind has been unbearable for three days."
  "It started on the night of the first and hasn't stopped to breathe. There is no rain behind it."
  "Strangely, I've been forgetting less."
  "I remember the names of all twenty-one children. I remember what I ate for lunch."
  "I remember that I am leaving."
  "I think I will stop the diary here. I wrote it to hold on to myself, and I no longer need to."
  "One month and four days, and I am gone."
  jump moriarty_the_lady

scene moriarty_the_lady:
  "August 9th. Something very strange happened yesterday."
  "A young woman came to the door after midnight, soaked through."
  "The wind had stopped that same afternoon."
  show mary tired
  "Pretty, and with the bearing of a noblewoman, though worn down to the bone."
  "She did not ask for discretion. She did not offer money. She did not cry."
  "She did everything like someone keeping an appointment."
  moriarty "What is his name?"
  mary "Gabriel."
  moriarty "And his family name?"
  "She looked at the door."
  mary "Don't let anyone come looking for me."
  "She asked for only one thing, and she asked twice, which struck me as excessive."
  remove mary
  "I decided to take him in."
  jump moriarty_the_men

scene moriarty_the_men:
  "August 11th. Men came this morning. Three of them, in black, polite in a way that makes you cold."
  show man_in_black neutral
  man_in_black "Good morning. Has a new child come in this week?"

  choice:
    "No":
      moriarty "No. No one new."
    "Tell the truth":
      "I open my mouth to tell the truth, and I hear myself say no."
      moriarty "No. No one new."

  man_in_black "Thank you for your time."
  remove man_in_black
  "I lied to an armed man before breakfast, and I do not recognise myself."
  "Afterwards I unwrapped the boy's cloths to change him, and there was"
  "..."
  "(The sentence ends mid-line. The next page has been torn out at the stitching.)"
  jump moriarty_the_toys

scene moriarty_the_toys:
  "August 15th. Caring for a baby is always hard. This one is particularly complicated."
  "Things keep appearing in his cot. He looks at a toy, and the next morning it is beside him."
  "One of the children must have found a way into the nursery at night, after we lock the wing."
  "August 20th. I have watched for days. Nobody comes. Nobody goes in."
  "And still the toys appear."
  "None of them belong to this house. I made the inventory myself in April."
  "They are not old toys. They are expensive toys."
  "August 29th. This baby has some unusual ability. I cannot prove it, but I am sure of it."
  "I will not write that anywhere but here."
  "September 2nd. It has stopped."
  "Maybe he has no control over it. Maybe I imagined it."
  jump moriarty_the_letter

scene moriarty_the_letter:
  "September 3rd. I wrote to my sister today."
  "I told her there had been a complication at the house..."
  "...and that I must stay until the end of the year."
  "There was no complication."
  "This morning I held the boy while he slept, and I understood something I'm not proud of."
  "If I leave, there will be no one left here who knows what I know."
  "(Last entry in the notebook. About sixty blank pages follow.)"
  jump moriarty_1901

scene moriarty_1901:
  # Part II of the annex -- transcript of the conversation, 1901
  clear
  "Seven years later. My office at Santa Ilde, 1901."
  show francis neutral
  "I push the notebook across the desk before he has even sat down."
  moriarty "Read it. Then you'll understand why I talk about that half-year the way I do."
  "He takes longer than he needs to. I fill two pipes and light neither."
  moriarty "I had started here four months before. First post. I didn't know a soul in town."
  moriarty "I thought about leaving more often than I care to admit."
  francis "Then why did you stay? After everything in there?"
  moriarty "I grew attached. To my colleagues. To Gabriel."
  moriarty "Look after a child for seven years and then tell me you can pack your bags."
  francis "The man whose name is scraped out."
  "I take my time."
  moriarty "I don't remember."
  moriarty "It isn't pretence, Francis. I open my mouth to describe him and nothing comes."
  moriarty "It doesn't feel like forgetting. It feels like he was never here."
  moriarty "The papers with his name were lost or burned, in separate accidents."
  moriarty "I scraped those words out with a razor and I have no idea why."
  francis "Strange that nobody investigated."
  moriarty "You know the orders from above. Nobody touches this."
  moriarty "The group that got too curious doesn't work here anymore. Or anywhere."
  francis "In one month a man arrives that the world forgets."
  francis "In the next, a child arrives with a peculiar signature. That isn't chance."
  "I stop the pipe halfway to my mouth at the word signature."

  choice:
    "Correct him":
      "Not a signature. I nearly say it. I weigh it, and I let it go."
    "Let it pass":
      "I let it pass. Let him think it was surprise."

  moriarty "Of course it's strange."
  moriarty "But Gabriel is what's pulling this house out of the hands it's walked with for thirty years."
  moriarty "You know where the children went before '94."
  moriarty "And above all that, he is still a child. Seven years old, Francis. Seven."
  francis "The torn page. What was in the boy's cloths?"
  moriarty "Nothing. Cloths."
  "The only quick answer of the night. It had been ready for years."
  francis "But..."
  moriarty "No 'but'. This conversation didn't happen."
  "At the door, my hand on the knob, I do not turn around."
  moriarty "You don't want to be the next one."
  moriarty "And Francis. Whoever it is you write to at night, tell them this."
  moriarty "There's no family here. There's a boy."
  remove francis
  jump post_start
