#  Übungsblatt 2
## Betriebssysteme WiSe 2020/21 Barry Linnert

Karl Skomski, Corin Baurmann

### Aufgabe 2-1 (Threads) – 6 Punkte

> Wann ist  es  sinnvoll,  nebenläufige  Programmteile  mit  Hilfe  von  Threads  anstatt  Prozessen  (heavyweight processes) zu implementieren?

Wenn innerhalb eines Programms mehrere Aufgaben existieren, die parallel abarbeitbar sind, dh. parallelisierbar sind und einen gemeinsamen Kontext, d.h. einen gemeinsamen Heap benutzen können/müssen, ist es sinnvoll, das Programm in Threads aufzuteilen. Die Kommunikation zwischen Threads ist wesentlich einfacher als zwischen Prozessen durch den gemeinsamen Adressraum. Durch den getrennten Adressraum zwischen Prozessen ist der Switch aufwendiger als bei Threads.

> Welche  Vorteile  bieten  User-Level-Threads  gegenüber  den  Kernel-Level-Threads?  Gibt  es  auch Nachteile?

Bei User-Level-Threads kann das Scheduling dieser Threads durch das runtimesystem kontrolliert werden. Das "Gewicht" des Threads im Bezug auf Speicherallokation, Rechenleistung und anderen Ressourcen kann durch das Runtimesystem innerhalb des Kernel-Level-Threads selber bestimmt werden.
Ein Nachteil dieses Modells ist ein komplizierterer Zugriff auf I/O, da ein User-Level-Thread die Schnittstellen des Betriebsystems nicht direkt nutzen kann, weshalb man normalerweise Eventloops benutzt, was den Umgang schwieriger macht. Je nach Implementation der User-Level-Threads und Anwendungsdomäne können die hardwareunterstützten Operationen des Betriebsystems effizienter sein als gleichwertige Operationen der Runtime.

> Welche  Vorteile  und  Nachteile  gibt  es,  wenn  man  Thread-Kontrollblöcke  (TCB)  als  Skalare,  in  Arrays, Listen, Bäumen oder invertierten Tabellen speichert?

Skalare, Arrays: statisch, aber ausreichend fuer waschmaschinen mit festen programmen
Listen: dynamisch, schlecht durchsuchbar O(n), einfügen/löschen einfach
Baeumen: dynamisch, leicht durchsuchbar O(log(n)), einfügen/löschen schwierig
Invertierte Tabellen: fuer den scheduler gut geeignet, da nach wichtigen  attributen sortiert, einfuegen mehrfache arbeit


> In   welchem   Adressraum   (Prozess-Eigner,   Dienste-Prozess,   BS-Kern)   wird   ein   TCB   gespeichert?

Wenn es sich um einen Kernel-Level-Thread handelt, wird der TCB im BS-Kern gespeichert. Wenn es sich um einen User-Level-Thread handelt, muss der Prozess-Eigner die TCBs in seinem vom BS zugewiesenen Memory speichern.


### Aufgabe 1-2

Github-Repository https://github.com/docweirdo/rOSt
Binary ist in der Repo (https://github.com/docweirdo/rOSt/tree/master/target/armv4t-none-eabi/debug/), lauffaehig auf dem Andorraserver.