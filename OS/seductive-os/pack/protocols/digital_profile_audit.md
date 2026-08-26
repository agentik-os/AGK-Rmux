# Protocol: Digital Profile Audit

An audit of the user's dating app profile and opening messages. Run consent_check.md before any messaging work involving a specific person.

This is the one area of the pack where a large part of the problem is genuinely technical and genuinely solvable. "Nobody replies" is usually photos and format, not the person. Say so early, because a user who believes an app result is a verdict on their worth is carrying a much heavier thing than the actual problem.

The counterweight, said just as early: an app is a filtered, low-information channel with brutal base rates for almost everyone. Its output is not evidence about how the user does with people in a room, and the two should never be treated as the same measurement.

## Steps
1. Set expectations against real base rates before assessing anything. Low reply rates are normal, most matches go nowhere for everyone, and this is a property of the channel rather than of the user.
2. Audit the photo set as a set, not as individual photos. Six to eight, and the set has a job to do (see the table below). One clear face photo taken from a normal distance in daylight is the load-bearing item, and its absence is the most common single fault.
3. Check what the set actually communicates: face, body in context without a performance of it, one real activity, one social photo that does not require guessing which person is the user, and one photo with a life in it. No group photo first, no sunglasses in the first, no heavy filter, no photo more than about two years old.
4. Audit the text for one thing only: does it give somebody a specific, easy thing to reply to. Specific and slightly odd beats polished and generic, because the generic version gives the other person nothing to hold.
5. Strip the negatives. Lists of what the user does not want, rules for messaging them, and weary complaints about the app all read as a preview of the interaction. This is the second most common fault.
6. Check honesty across the whole surface: current photos, real height if stated, actual situation, and what the user is genuinely looking for. Concealing a partner, an STI status, or a fundamental fact of identity is refused outright, and beyond the ethics it is a promise that comes due on the first meeting.
7. Audit openers. One message, specific to something in their profile, ending somewhere they can go. Not "hey". Not a copy-paste sent at volume, which efficiently generates rejection, which is the exact resource the user has least tolerance for.
8. Set the message rule and make it structural: one unanswered message is an answer, two is pressure. There is no third. Write the rule down now, because it is much harder to hold in the moment.
9. Set the transition rule. Move to a real meeting within about a week of steady conversation, or let it go. Long text relationships build an imagined person on both sides, and the meeting then has to survive the gap between the imagined one and the real one.
10. Cap the changes at three, ordered by effect over cost. The photo set is almost always first.
11. Set a volume ceiling and a review date. Endless swiping is not practice, and app time that displaces real-world reps is the failure mode this protocol most often uncovers.
12. Log it as a `self_presentation_audit` record with channel `digital`, taste items marked **[P]** and owned by the user.

## What the photo set has to do
| Slot | Job | Common fault |
| --- | --- | --- |
| First | One clear, current, unobstructed face, daylight, normal distance | Sunglasses, a group, a filter, or a photo from five years ago |
| Second | The body in an ordinary context, no performance of it | Either hidden entirely or the only subject |
| Third | One real activity the user actually does | An activity done once, for the photo |
| Fourth | One social photo where the user is instantly identifiable | Six people, no clue which is the user |
| Fifth and sixth | A life with texture in it: a place, work, something made, something cared for | Repetition of the same angle and the same expression |
| Throughout | Consistency, so the person at the table is recognisably the person in the set | A set that promises someone else |

## Stop rules
- No mass messaging, no copy-paste at volume, no spray and pray. It treats people as a funnel, it degrades the space for everyone in it, and twenty considered messages beat four hundred copies on every metric including total matches.
- No unsolicited sexual images, in any framing. It is a violation and separately it is criminal in many jurisdictions.
- No impersonation, no photo that is not the user, no invented biography, no borrowed lifestyle.
- Do not research a match beyond what they offered. Searching an employer, an address, a family or an old account is surveillance, and it surfaces.
- Never screenshot, forward or share a private message or a profile for commentary. The person is not in the room.
- A stopped reply is an answer, complete in itself, requiring no follow-up to confirm. Route to post_rejection_debrief.md rather than to a re-engagement message.
- If app use has become compulsive, if the user is measuring their worth in matches, or if the swiping is displacing every real-world rep, stop the audit. Name it, cap the volume, and route to ../references/safety-and-boundaries.md under **C** where distress is present.
- Never store a third party's profile, handle, photo or messages in any record. The schemas have no field for them.

## Required closure
- Decision or output: three changes maximum, ordered by effect over cost, plus the message rule, the transition rule and the volume ceiling.
- Owner: the user owns every taste and photo decision.
- Observable completion evidence: a `self_presentation_audit` record with channel `digital` (see ../schemas/self_presentation_audit.json), and the changes actually applied to the live profile.
- Review trigger: three to four weeks after the changes land, judged on conversations that became meetings rather than on match count.
- Memory and handoff instruction: persist the user's own profile decisions only, with consent, never a third party's material. Route photos and grooming to self_presentation_audit.md, first meetings to date_design.md, and worth measured in matches to Mindset OS.
