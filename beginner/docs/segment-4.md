# Segment 4 detailed instructions

This segment pushes students to take turn drawing on the whole screen. The screen will rotate between participants, clearing the screen between each turn. If they wait patiently, they will be allowed to draw on the whole screen at a high rate. If they send messages when it is not their turn, they will be rate-limited in the number of messages they send.

The turns will be shown on the presenter's projector screen, and a radio message will be sent at the change of each turn.

## Instructor Notes

Your instructor will demonstrate how the screen will "take turns" for each participant. Each participant will have a 5 second window to draw on the full board.

Each turn will be announced in two ways:

1. The display will show the turn of each participant on the screen, allowing the student to see when it is their turn
2. The display will periodically announce the turn over the radio.

If your device sends messages outside of your turn, your maximum message rate will be reduced for the next turn! Be careful to only send messages during your turn.

During your turn, your maximum message rate will be increased to 128 messages per second, or one message per 7.8ms.

Students are free to implement one or more of the following strategies for drawing on the screen, increasing in difficulty:

## Strategy One: Manual Control using the debugger

You may load your program to the device, and wait to begin running code until your turn. Once your turn ends, you may stop the program using the debugger.

This requires the student to manually start and stop their program during their turn.

## Strategy Two: Use the on-board button

You may instruct your program to read the button, and wait to send messages until the button is pressed. Once the button is pressed, your program should continue until the end of your turn (5 seconds).

This requires the student to manually start their program during their turn.

## Strategy Three: Measure time locally

You may use the on-board timers to start sending messages, then wait an appropriate amount of time until the next turn.

This requires the student to manually start their program once, then drawing will continue automatically. Be careful to make sure your program does not "drift" into another student's turn!

## Strategy Four: Listen for your Turn

You may listen for announcements of which turn it is. These messages are sent out periodically. When it is your turn, you may begin sending messages for a fixed amount of time.

This requires students to alternate between only-sending and only-receiving messages.

## Strategy Five: Alternating TX/RX

You may alternate between sending and receiving messages, listening for an announcement whenever you are waiting for the the next time to send messages (e.g. send a message, then listen for the next 7.8ms or so).

This requires students to continuously alternate between sending and receiving messages.
