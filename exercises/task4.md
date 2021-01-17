#  Übungsblatt 4
## Betriebssysteme WiSe 2020/21 Barry Linnert

Karl Skomski, Corin Baurmann

## Aufgabe 4-1 (Nachrichten unterschiedlicher Länge) – 5 Punkte

>Im Allgemeinen sind an Kommunikationsobjekte Nachrichten unterschiedlicher Länge zugelassen.
> 1. Welche Auswirkungen hat dies auf die Daten des Kommunikationsobjekts? Bei welchen Varianten der Nachrichtenübergabe hat dies weitere Auswirkungen, und bei welchen hat dies sonst keine weiteren Auswirkungen?

Da bei Nachrichtenübermittlung by reference und by mapping im Kommunikationsobjekt nur die Addresse des Nachrichtenbuffers bzw. die des Zielbuffers gespeichert wird, hat bei diesen beiden Varianten die Nachrichtengröße keine Auswirkungen auf die Daten des Kommunikationsobjekts.  
Bei Transfer by value wird die Nachricht selber in und aus dem Kommunikationsobjekt kopiert. Damit hängt die Größe des Kommunikationsobjektes von der maximal zulässigen Nachrichtenlänge ab. Ein "statisches" Objekt ist dabei mindestens so groß wie die max. zul. Nachrichtenlänge, oder das Kommunikationsobjekt muss in der Lage sein inmitten eines Kopiervorgangs neuen Speicher zu allozieren. Die Nachrichtenlänge muss zusammen mit der Nachricht an das Kommunikationsobjekt übergeben werden.

> 2. Welche Probleme können aus der Sicht der Kommunikationspartner auftreten, wenn Nachrichten unterschiedlicher Länge zugelassen sind? Schlagen Sie geeignete Lösungen vor!

Der Empfänger ist sich bei variabler Nachrichtenlänge vor Empfang der tatsächlichen Nachrichtenlänge nicht bewusst. Bevor die Nachricht kopiert wird muss der Empfänger also dynamisch Speicher allozieren oder ein Zwischenschritt muss dem Empfänger die Größe der Nachricht vermitteln.
Zusaetzlich kann das Kommunikationsobjekt vielleicht nur eine maximale Nachrichtengröße unterstützen, wodurch man Fragmentierung auf beiden Seiten unterstützen muss.

### Aufgabe 4-2 (Kontextwechsel und präemptives Multitasking) – 15 Punkte

Github-Repository https://github.com/docweirdo/rOSt mit kurzem Setup lauffähig auf dem Andorraserver.
Nach Kernelstart `task4` eingeben, um die geforderte Anwendung zu starten.