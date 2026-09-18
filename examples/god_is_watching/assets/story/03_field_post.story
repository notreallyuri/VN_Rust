# God Is Watching -- Chapter 3: Field Reports
# POV: the unnamed head of the House's field post at Santa Ilde
# Source: "Field reports -- Santa Ilde" (Reports No. 203 to 221)

scene post_start:
  # Report 203
  "Field post, Santa Ilde. Internal correspondence. It does not leave the House."
  "Report No. 203."
  "Santa Ilde has owed us servants since 1871."
  "In 1898 the numbers fell by half. In 1899 they stopped."
  "In the same period the shipments to Verlaine, Oduard and Sant'Anne stopped as well."
  "A house that breaks with one House is in trouble. A house that breaks with four is being carried."
  "My guess is a lesser House, trying to take the tap instead of fighting over the water."
  jump post_the_master

scene post_the_master:
  # Report 206
  "Report No. 206. Two months, and no answers."
  "Ruled out: the administrator was not bought."
  "He lives on his wage and has worn the same coat for seven years."
  "Ruled out: church protection. The bishopric doesn't even know how many children live there."
  "Ruled out: plain coercion."
  "A note arrives from the House, for the younger men who still mix up the terms."
  show house_envoy neutral
  house_envoy "A signature is learned. A blessing is inherited."
  house_envoy "If there is a hand at work in Santa Ilde, someone trained it. Every training leaves a master."
  house_envoy "Find the master and you find the hand."
  remove house_envoy
  "We are looking for the master."
  jump post_safeguards

scene post_safeguards:
  # Report 208
  "Report No. 208. We have put a man inside the orphanage again."
  "The last one left on his own. He asked to be removed from service and gave no reason."
  "His request is four lines long and has three wrong dates in it."
  "The house has started using safeguards. Wings locked after the bell."
  "The entry register rewritten in January."
  "It must be a House. A provincial orphanage doesn't invent that kind of discipline by itself."

  choice:
    "Look at what they protect":
      "The safeguards cover the nursery and the young children's wing. Not the ledgers. Not the safe."
    "Look at what they leave open":
      "The ledgers sit in an unlocked cabinet. The safe has no guard. The nursery has three."

  "Whoever gives the orders there is afraid for people, not for paper."
  jump post_the_shadows

scene post_the_shadows:
  # Report 210
  "Report No. 210. We have reached the limit of what ordinary men can do."
  "We need someone who can trace the use of signatures."
  "I formally request the House's shadows."
  "I know what I am asking for, and what it costs. I ask anyway."
  "A year and a half, circling a stone house with twenty-one children inside."
  "And not one name to send home."
  jump post_annex

scene post_annex:
  # Report 215 -- reading of the annex to 214
  "Report No. 215. Francis's annex arrives: the administrator's notebook, and a conversation from 1901."
  show francis neutral
  francis "He handed me the notebook himself."
  francis "I copied it in one night and returned it before the six o'clock bell."
  francis "He doesn't know there's a copy."
  "I read it twice. Three things bother me, in this order."
  "First. He suspects our man, and he handed over the notebook anyway."
  "No man hands evidence to a spy he has already spotted, unless he wants it read."
  "We may have been fed."
  "Second. Francis says the administrator flinched at the word signature. Francis calls it surprise."
  "I am not so sure."
  "Third. There is a torn page, and a ready-made answer about what was on it."
  "What was wrapped in those cloths on the 11th of August, 1894? That comes before everything else."
  francis "And his last words? No family, only a boy?"

  choice:
    "Underline it":
      "I reach for the pen to underline it. Then I write in the margin instead."
    "Dismiss it":
      "I write in the margin without hesitating."

  "A diversion by the administrator. Not to be treated as information."
  remove francis
  "(A later note, in different ink, in the same hand.)"
  "I read the paragraph above again in January. I leave it as it is."
  "Let it stand on record how wrong we were, and for how long."
  jump post_residue

scene post_residue:
  # Report 217
  "Report No. 217. The shadows arrived on the eleventh. There are three."
  "I did not ask their names, and none were offered."
  show shadow neutral
  "They work for four nights. I write down what they tell me, in order, without interpreting."
  shadow "There is residue in Santa Ilde, and a great deal of it. In the young children's wing."
  shadow "The residue has no master."
  "I ask what that means."
  shadow "Every trained hand carries the gesture of whoever trained it, like handwriting."
  shadow "There is no trace here. It is as if someone were writing without ever having seen a letter."
  remove shadow
  "The third shadow asks to sweep the old wing, closed since '94."
  show shadow_third neutral
  "He comes back in the morning, sits at the table, and asks why we are in Santa Ilde."
  "I repeat the assignment. He writes it down."
  shadow_third "Why are we in Santa Ilde?"
  "Half an hour later. The same question."
  remove shadow_third
  "I send him back to the House that afternoon."
  "Neither of the other two remembers that there were three of them."
  jump post_von_lucis

scene post_von_lucis:
  # Report 219
  "Report No. 219. The cloths. It is not in Santa Ilde."
  "The shadows swept the administrator's lodgings, the sacristy, the infirmary floor. Nothing."
  "So we go to the Von Lucis residence, the likely origin of the child according to Report 212."
  "We are late. The house has been shut for years, and someone was here before us. Someone methodical."
  "Folio 41 of the register has been torn out."
  "The floorboard by the fireplace in the ground-floor room is loose..."
  "...and the space beneath it is empty."
  "Whatever was kept there left in one piece, without breaking anything. Not a looting. A collection."
  "Someone is putting together the same story we are, and is three steps ahead."
  "It is not the orphanage. The orphanage doesn't know there is a room with a loose floorboard."
  jump post_blessing

scene post_blessing:
  # Report 220
  "Report No. 220. I ask that the House read this in full before replying."
  "There is no use of signatures in Santa Ilde. No master, no hand, no rival House."
  "We spent two years looking for a master becuse Report 206 told us to."
  "Report 206 was right about the rule and wrong about the case."
  "Whoever does this has a blessing. A blessing is inherited, and inheritance has blood."
  "The blood there is Von Lucis."
  "And whoever does it has no idea what he is doing."
  "I withdraw the instruction of 215."
  "The administrator's last sentence was not a diversion. It was an answer."
  "He told us out loud, in 1901. No family. A boy."
  "We filed it as a distraction, and spent two more years and three shadows."
  "One did not come back whole."
  "He was seven when that conversation took place. He is nine now."
  "One last observation, and it is mine, not the shadows'."
  "If the child does not know what he does, he did not choose to break with any House."
  "A child does not decide to free an orphanage."
  "Someone decided for him, and has been deciding since 1894."
  "And that person knows exactly what they hold."
  "I await instructions. I recommend, as strongly as I can, that they are not instructions to approach."
  jump post_last

scene post_last:
  # Report 221 -- the document breaks off here
  "Report No. 221."
  "(Hurried handwriting. The ink is smeared across two lines.)"
  "One of the shadows claims to have discov"
