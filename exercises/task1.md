#  Übungsblatt 1
## Betriebssysteme WiSe 2020/21 Barry Linnert

Karl Skomski, Corin Baurmann

### Aufgabe 1-1 (Architekturen) – 5 Punkte
Vorhandene Betriebssystementwürfen können grob in zwei Klassen eingeteilt werden:- die Makrokernarchitektur,- die Mikrokernarchitektur.

Diskutieren Sie beide Architekturansätze unter Zuhilfenahme mindestens folgender Quellen:
- J. Liedtke, Toward real μ-kernels, Communications of the ACM, 39(9):70--77, September 1996, http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.40.4950&rep=rep1&type=pdf
- C. Maeda, B.N. Bershad, Networking performance for microkernels, Proceedings of Third Workshop on Workstation Operating Systems, 13:154 – 159, April 1992 http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.56.9733&rep=rep1&type=pdf

### Makrokernarchitektur

 #### Leitidee: Geschwindigkeit und weniger Kommunikationsoverhead
 + Einfacher zu debuggen
 + Weniger Kopierarbeit in versch. Adressräume
 - Treiber bringen einfach das gesamte System zum Absturz bei Fehlern
 - Bei Veraenderungen muss immer der gesamte Kernel neu kompiliert werden


### Mikrokernarchitektur

 #### Leitidee: Den Kernel so klein wie moeglich zu halten, modularer Aufbau
 + Höhere Flexibilität
 + Einfacher zu debuggen und zu modifizieren
 + Treiber sind im Userspace und dadurch mehr Ausfallsicherheit gewaehrleistet
 + Einfacher neuzuladen


 ### Artikelspezifisch
  + Am Anfang oft Schwierigkeiten, wenn man Makrokernel versucht in Mikrokernel umuzwandeln -> oft grundlegende strukturelle Veraendungeren noetig, um gleiche Performance zu bieten
  + Microkernel muessen oft fuer den Prozessor speziell optimiert werden, damit die grundlegenden IPC-Funktionen schnell genug sind, um mit Makrokernels zu konkurrieren
  + Kerneltreiber aus Makrokernarchitektur kann man nicht einfach in den Userspace in Mikrokerneln packen, da durch die Kommunikation durch IPC andere Programmierphilosophien noetig sind siehe Beispiel UDP-Stack
  + Mikrokernel selber sind selber wenn sie effizient implementiert sein sollen nicht wirklich portierbar und müssen an die jeweilige Plattform angepasst werden. 
  + Zu der Zeit der Artikel waren Mikrokernel noch wenig im Einsatz.  24 Jahren spaeter scheinen Mikrokernel eher immer noch ein Schattendasein zu fuehren. Warum ist das so bzw. was macht RIOT anders, um die obigen Schwachstellen auszubuegeln? Weitere Recherche notwendig.


### Aufgabe 1-2

Wir haben Ihnen eine Einladung zu unserer Github-Repository https://github.com/docweirdo/rOSt per Email geschickt.