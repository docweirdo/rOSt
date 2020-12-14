#  Übungsblatt 3
## Betriebssysteme WiSe 2020/21 Barry Linnert

Karl Skomski, Corin Baurmann

## Aufgabe 3-1 (Threads) – 6 Punkte

### Gegenseitiger Ausschluss

Gegenseitiger Ausschluss bedeutet die exklusive Nutzung einer Ressource durch einen Teilnehmer. Der gegenseitige Ausschluss muss über ein technisches oder logisches Konstrukt sichergestellt werden, wenn für eine bestimmte Nutzung der Ressource exklusive Nutzung verlangt wird, um die Determiniertheit des Systems sicherzustellen. Diese bestimmte Nutzung einer Ressource wird kritischer Abschnitt genannt.

### Signalisierung 

Das Ziel der Signalisierung ist die zeitliche Anordnung von Aktivitäten festzuschreiben. Sie stellt ein Werkzeug dar, um verschiedene Teilnehmer, z.B. Threads, zu koordinieren. Damit stellen Signale eine Form der Thread Interaktion dar, die sich auf den zeitlichen Aspekt bezieht.

### Synchronisation

Synchronistation ist eine Form der Koordination, d.h. der zeitlichen Interaktion. Vorgänge sollen gleichzeitig bzw. in einer bestimmten Reihenfolge stattfinden.

### Koordination

Koordination ist die eine Interaktion zwischen Threads, die auf einen zeitlichen Aspekt abzielt. Für Kommunikation und Kooperation wird eine Koordination als Grundlage benötigt. Die Koordination kann über Werkzeuge wie Sinale bewerkstelligt werden.

### Kommunikation

Koomunikation ist der direkte Austausch zwischen Teilnehmern. Es gibt asynchrone und synchrone Kommunikation, wobei beim letzteren die zeitliche Koordination zusätzlich vorhanden sein muss.

### Kooperation

Kooperation ist der gemeinsamene Zugriff von Teilnehmern auf Daten und stellt eine Form der Interaktion mit Bezug auf den funktionalen Aspekt dar. Für gemeinsamen Zugriff auf geteilte Daten wird Koordination zwischen den Teilnehmern benötigt. 


### Aufgabe 3-2 (Interrupts – System Timer und serielle Schnittstelle) – 15 Punkte

Github-Repository https://github.com/docweirdo/rOSt mit kurzem Setup lauffähig auf dem Andorraserver.
Nach Kernelstart `task3` eingeben, um die geforderte Anwendung zu starten.