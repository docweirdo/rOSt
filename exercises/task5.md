#  Übungsblatt 5
## Betriebssysteme WiSe 2020/21 Barry Linnert

Karl Skomski, Corin Baurmann

## Aufgabe 5-1 (Gerätezugriff) – 5 Punkte

>Erläutern Sie die Begriffe:  
>− memory mapped I/O,  
>− DMA (direct memory access) und  
>− polling.  

### Memory mapped I/O
Das bedeutet, dass Input/Output Devices, also Peripherie und diverse Schnittstellen um Geräte anzuschließen, in den logischen Adressraum gemapped werden. Verschiedene Register zur Korrespondenz mit der Peripherie können also über eine Adresse angesprochen werden, als läge die Information im Arbeitsspeicher.

### DMA
Direct memory access ist ein Konzept, welches den Zugriff auf den Arbeitsspeicher ohne CPU Betätigung ermöglicht. Normalerweise übernimmt ein konfigurierbarer DMA Controller diese Aufgabe. So können unter Entlastung der CPU z.B. Daten aus der Peripherie in den Arbeitsspeicher geladen werden.

### Polling
Beim Polling wird eine Information kontinuierlich von der CPU abgefragt. Dabei kann es sich um Daten eines Peripheriegeräts handeln, den Zustand einer Ressource oder irgendeine andere Information, deren Veränderung über einen gewissen Zeitraum Relevanz hat. 

>Gegeben sei nun ein Prozessor mit einer Taktfrequenz von 200 MHz. Eine Polling-Operation benötige 400 Taktzyklen. Berechnen Sie die prozentuale CPU-Auslastung durch das Polling für die folgenden Geräte. Bei 2 und 3 ist dabei die Abfragerate so zu wählen, dass keine Daten verloren gehen.
>1. Maus mit einer Abfragerate von 30/sec,
>2. Diskettenlaufwerk: Datentransfer in 16-bit-Wörtern mit einer Datenrate von 50 KB/sec,
>3. Plattengerät, das die Daten in 32-bit-Wörtern mit einer Rate von 2 MB/sec transportiert.

1. 0.01 %
2. 5%
3. 100%

> Für das Plattengerät soll jetzt DMA eingesetzt werden. Wir nehmen an, dass für das Initialisieren des DMA 4000 Takte, für eine Interrupt-Behandlung 2000 Takte benötigt werden und dass per DMA 4 KB transportiert werden. Das Plattengerät sei ununterbrochen aktiv.
Zugriffskonflikte am Bus zwischen Prozessor und DMA-Steuereinheit werden ignoriert.
>
>Wie hoch ist nun die prozentuale Belastung des Prozessors? (Hinweise: Wie viele DMATransfers pro Sekunde sind bei 2 MB/sec und 4 KB Transfergröße notwendig?)

(4000 + 500 * 2000) von 200 MHz = 0.5%

### Aufgabe 5-2 (User-/Kernel-Interface) – 15 Punkte

Github-Repository https://github.com/docweirdo/rOSt mit kurzem Setup lauffähig auf dem Andorraserver.
Nach Kernelstart `task5` eingeben, um die geforderte Anwendung zu starten.