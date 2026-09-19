# God Is Watching -- Chapter III: Field Reports
# The House's field post at Santa Ilde, reports 203 to 221 (1899-1903).

scene reports_start:
  background field_post
  music santa_ilde
  "Report No. 203. Santa Ilde has owed us servants since 1871. In 1898 the numbers fell by half. In 1899 they stopped."
  "A house that breaks with one House is in trouble. A house that breaks with four is being carried."
  "Report No. 206. A note from the House, for the younger men who still mix up the terms."
  show house_envoy neutral at center
  house_envoy "A signature is learned. A blessing is inherited."
  house_envoy "If there is a hand at work in Santa Ilde, someone trained it. Find the master and you find the hand."
  remove house_envoy
  "We are looking for the master."
  jump reports_the_shadows

scene reports_the_shadows:
  "Report No. 217. The shadows arrived on the eleventh. There are three."
  show shadow neutral at left
  shadow "There is residue in Santa Ilde, and a great deal of it. In the young children's wing."
  shadow "The residue has no master. It is as if someone were writing without ever having seen a letter."
  show shadow_third neutral at right
  "The third shadow sweeps the old wing, closed since '94. He comes back in the morning and asks why we are in Santa Ilde."
  shadow_third "Why are we in Santa Ilde?"
  "Half an hour later. The same question."
  clear
  "Neither of the other two remembers that there were three of them."
  jump reports_von_lucis

scene reports_von_lucis:
  background von_lucis_hall
  music von_lucis
  "Report No. 219. We went to the Von Lucis residence. We are late. Someone methodical was here before us."
  "Folio 41 of the register has been torn out. The floorboard by the fireplace is loose, and the space beneath it is empty."
  if saw_torn_page == true:
    "You stop reading. Folio 41. You have seen that number before, pressed into a page from 1894."
  "Someone is putting together the same story we are, and is three steps ahead."
  jump reports_blessing

scene reports_blessing:
  background field_post
  music santa_ilde
  "Report No. 220. There is no use of signatures in Santa Ilde. No master. No rival House."
  "Whoever does this has a blessing. A blessing is inherited, and inheritance has blood. The blood there is Von Lucis."
  "And whoever does it has no idea what he is doing."
  "If the child does not know what he does, someone decided for him, and has been deciding since 1894."
  "And that person knows exactly what they hold."
  jump reports_last

scene reports_last:
  "Report No. 221."
  "(Hurried handwriting. The ink is smeared across two lines.)"
  background none
  "One of the shadows claims to have discov"
  "..."
  background archive_office
  music archive
  show registrar stern at right
  registrar "Well?"
  if approach == bold:
    registrar "You wanted to go and ask them. Go and ask them."
  else:
    registrar "You've read it all. Now go and look at it."
  registrar "Santa Ilde. Ask for the administrator. Don't ask for the boy."
  remove registrar
  jump ilde_start
