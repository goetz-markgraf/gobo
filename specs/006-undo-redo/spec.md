# Feature Specification: Undo / Redo

**Feature Branch**: `006-undo-redo`

**Created**: 2026-07-02

**Status**: Draft

**Input**: User description: "Undo-Feature Ich möchte, dass mit Ctrl-Z ein Undo genutzt werden kann und mit Ctrl-Y ein Redo. Jede Textänderung (jetzt und in Zukunft) soll undoable sein. Der Undo-Stack ist prinzipiell unendlich -- das Ende ist erst erreicht, wenn der Speicher ausläuft. Wenn man die Anwendung schließt, ist der Undo-Stack entfernt. Auch der Redo-Stack ist unendlich. Aber bei einer Änderung wird der Redo-Stack geleert."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Mistakes rückgängig machen (Priority: P1)

Ein Nutzer tippt Text ein und macht dabei einen Fehler oder entscheidet sich um. Er drückt Ctrl-Z und der letzte Textbeitrag wird zurückgenommen, sodass der Textzustand genau wiederhergestellt ist, der vor der letzten Änderung bestand. Wiederholtes Drücken von Ctrl-Z geht Schritt für Schritt weiter in vergangene Zustände zurück.

**Why this priority**: Undo ist die zentrale Sicherheitsfunktion eines Editors. Ohne sie ist jeder Tippfehler potenziell destruktiv. Diese Reise liefert bereits für sich allein einen nutzbaren Mehrwert und ist die Grundlage für alle weiteren Stories.

**Independent Test**: Kann vollständig getestet werden, indem ein Leer- oder Seed-Dokument geöffnet, mehrere Änderungen vorgenommen und dann durch wiederholtes Ctrl-Z wieder der Ausgangszustand erreicht wird. Erwartete Wirkung: der Textinhalt stimmt an jedem Punkt mit dem jeweils vorherigen Zustand überein.

**Acceptance Scenarios**:

1. **Given** ein offenes, leeres Dokument, **When** der Nutzer "a", "b", "c" eintippt und dann dreimal Ctrl-Z drückt, **Then** ist das Dokument wieder leer.
2. **Given** ein Dokument mit Text "Hallo", **When** der Nutzer das letzte Zeichen löscht und Ctrl-Z drückt, **Then** steht "Hallo" wieder im Dokument.
3. **Given** der Nutzer hat nacheinander mehrere Änderungen vorgenommen, **When** er Ctrl-Z k-fach drückt (k ≤ Anzahl der Änderungen), **Then** entspricht der angezeigte Text genau dem Zustand nach der (n−k)-ten Änderung.

---

### User Story 2 - Änderungen wiederherstellen (Priority: P2)

Nachdem der Nutzer per Ctrl-Z einen oder mehrere Schritte zurückgegangen ist, kann er mit Ctrl-Y die zurückgenommenen Änderungen wiederherstellen. Jeder Druck auf Ctrl-Y stellt genau einen Schritt wieder her, und zwar in derselben Reihenfolge, in der sie zuvor rückgängig gemacht wurden.

**Why this priority**: Redo ergänzt Undo zu einem vollständigen Verlauf. Es verhindert, dass versehentlich zu weit zurückgenommene Schritte verloren gehen, und macht den Verlauf navigierbar.

**Independent Test**: Kann getestet werden, indem Änderungen vorgenommen, per Ctrl-Z einige Schritte zurückgegangen und dann per Ctrl-Y wiederhergestellt werden. Erwartete Wirkung: der Text kehrt Punkt für Punkt zum zuletzt geänderten Zustand zurück.

**Acceptance Scenarios**:

1. **Given** der Nutzer hat dreimal Ctrl-Z gedrückt (drei Schritte zurück), **When** er dreimal Ctrl-Y drückt, **Then** steht der Text wieder im zuletzt geänderten Zustand.
2. **Given** der Nutzer ist durch Undo im Ausgangszustand angekommen, **When** er Ctrl-Y drückt, **Then** wird der erste Undo-Schritt wiederhergestellt.
3. **Given** alle rückgängig gemachten Schritte sind per Redo wiederhergestellt, **When** der Nutzer erneut Ctrl-Y drückt, **Then** ändert sich nichts weiter (Redo-Stack ist leer).

---

### User Story 3 - Redo wird bei neuer Änderung geleert (Priority: P2)

Hat der Nutzer per Ctrl-Z Schritte zurückgenommen und neue Änderung vorgenommen (Text getippt, gelöscht o. ä.), dann sind die zuvor rückgängig gemachten Schritte nicht mehr per Redo erreichbar. Der Redo-Verlauf wird bei jeder neuen Änderung verworfen, während der Undo-Verlauf die neue Änderung als obersten Eintrag erhält.

**Why this priority**: Diese Regel schützt vor inkonsistenten Verläufen. Sie ist eine Kernaussage der Feature-Beschreibung und sicherheitsrelevant, da sie unerwartete Textzustände nach Mischen von Undo und Neu-Eingabe verhindert.

**Independent Test**: Kann getestet werden, indem eine Änderungskette aufgebaut, ein Schritt per Undo zurückgenommen, dann eine neue Änderung vorgenommen und geprüft wird, ob Ctrl-Y keine Wirkung mehr zeigt.

**Acceptance Scenarios**:

1. **Given** der Nutzer hat "a", "b" eingetippt, einmal Ctrl-Z gedrückt (nur "a" übrig), **When** er nun "x" eintippt, **Then** ist der Text "ax" und ein anschließendes Ctrl-Y bewirkt nichts (Redo-Stack leer).
2. **Given** mehrere Undo-Schritte wurden ausgeführt und der Nutzer nimmt eine neue Änderung vor, **When** er danach Ctrl-Y drückt, **Then** findet keine Wiederherstellung der alten Schritte statt; stattdessen kann nur die neue Änderung per Ctrl-Z rückgängig gemacht werden.

---

### User Story 4 - Ein Leben pro Anwendungssitzung (Priority: P3)

Der Undo-Verlauf beginnt mit dem Öffnen des Dokuments und endet beim Schließen der Anwendung. Es wird nichts auf die Festplatte geschrieben; beim Neustart ist der Verlauf leer, und das Dokument startet mit einem frischen Verlauf.

**Why this priority**: Diese Abgrenzung hält das Feature einfach und lokal. Sie klärt Lebensdauer und Speicherbezug und verhindert Missverständnisse über Persistenz.

**Independent Test**: Kann geprüft werden, indem nachgewiesen wird, dass beim Schließen und erneuten Öffnen der Anwendung kein früherer Verlauf mehr vorhanden ist (auch nicht bei einem ungespeicherten oder gespeicherten Dokument).

**Acceptance Scenarios**:

1. **Given** der Nutzer hat in einer Sitzung einen Verlauf aufgebaut und die Anwendung geschlossen, **When** er die Anwendung erneut startet und dieselbe Datei öffnet, **Then** sind Ctrl-Z und Ctrl-Y ohne Wirkung, weil die Verläufe leer sind.
2. **Given** das Dokument wird gespeichert und die Sitzung beendet, **When** das Dokument in einer neuen Sitzung geöffnet wird, **Then** existiert nur die Möglichkeit, ab sofort vorgenommene neue Änderungen rückgängig zu machen, nicht jedoch frühere aus der vorherigen Sitzung.

---

### Edge Cases

- **Undo am Anfang des Verlaufs**: Ist der Undo-Stack leer (keine Änderung mehr rückgängig zu machen), bleibt ein Druck auf Ctrl-Z wirkungslos; der Text ändert sich nicht, die Anwendung bleibt stabil.
- **Redo am Ende des Verlaufs**: Ist der Redo-Stack leer, bleibt Ctrl-Y wirkungslos; der Text ändert sich nicht.
- **Sehr lange Verläufe**: Es wird künstlich keine Obergrenze gesetzt; die Verläufe wachsen mit, bis der verfügbare Speicher aufgebraucht ist. In diesem Grenzfall wird kontrolliert der älteste Undo-Schritt verworfen, um Platz zu schaffen, ohne bestehende Verläufe oder den Dokumenttext zu beschädigen (siehe FR-006).
- **Mischen von Undo und Neu-Eingabe**: Nach einer neuen Änderung sind vorher per Undo zurückgenommene Schritte nicht per Redo erreichbar; es entstehen keine unbestimmten Zwischenzustände.
- **Große einzelne Änderungen**: Eine sehr große Einfüge- oder Löschoperation muss als ein einzelner Undo-Schritt behandelbar sein, ohne die Anwendung beim Undo/Redo lahmzulegen.
- **Unicode-/Mehrzeil-Änderungen**: Auch Eingaben und Löschungen, die Zeilenumbrüche oder mehrbyteige Zeichen betreffen, müssen korrekt rückgängig und wiederherstellbar sein.
- **Undo/Redo während einer Eingabeaufforderung**: Undo und Redo wirken ausschließlich im Bearbeitungsmodus; in Popup-/Such-/Bestätigungsmodi werden Ctrl-Z und Ctrl-Y ignoriert, sodass keine unbestimmtenübergänge in diesen Modi entstehen.
- **Speichererschöpfung**: Wenn beim Aufnehmen eines neuen Schritts der Speicherplatz nicht ausreicht, muss der Editor das sicher abfangen (siehe FR-006), ohne den aktuellen Text zu zerstören: der älteste Undo-Schritt wird verworfen, um Platz zu schaffen; ist auch das nicht möglich, wird der Nutzer informiert und die Eingabe bleibt unbeschadet.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUSS eine Undo-Funktion bereitstellen, die durch Ctrl-Z ausgelöst wird und die zuletzt vorgenommene Textänderung in einen einzigen Schritt zurücknimmt.
- **FR-002**: System MUSS eine Redo-Funktion bereitstellen, die durch Ctrl-Y ausgelöst wird und den zuletzt per Undo zurückgenommenen Schritt wiederherstellt.
- **FR-003**: System MUSS jede Textänderung – einzeln und wiederholt – rückgängig machen können, einschließlich Einfügen einzelner Zeichen, Einfügen mehrerer Zeichen, Löschen (Rücktaste/Entfernen), Zeilenumbrüchen und Ersetzungen.
- **FR-004**: Der Undo-Verlauf MUSS theoretisch unbegrenzt sein; eine künstliche Obergrenze darf nicht gesetzt werden. Das Ende ist erst erreicht, wenn der verfügbare Speicher aufgebraucht ist.
- **FR-005**: Der Redo-Verlauf MUSS ebenfalls theoretisch unbegrenzt sein; auch hier gilt als Ende erst der Speichermangel.
- **FR-006**: System MUSS den Grenzfall der Speichererschöpfung beim Aufnehmen eines neuen Verlaufsschritts sicher behandeln: bestehende Verläufe und insbesondere der aktuelle Dokumenttext dürfen dabei nicht beschädigt werden. Bei Speichermangel MUSS der älteste Undo-Schritt verworfen werden, um Platz für den neuen Schritt zu schaffen, so dass die neue Textänderung weiterhin aufgezeichnet wird. Die Textänderung wird in jedem Fall ausgeführt (kein Blockieren der Eingabe). Ist auch nach Verwerfen des ältesten Schritts oder generell bei Aufnahmefehler kein Platz verfügbar, MUSS der Nutzer über eine Status-/Fehlermeldung informiert werden; die Änderung geht nicht stillschweigend verloren. Die Geschichte bleibt damit praktisch unbegrenzt, mit dem ältesten Eintrag als erstes Opfer bei Speichermangel.
- **FR-007**: System MUSS den Redo-Verlauf vollständig leeren, sobald nach einem Undo eine neue Textänderung erfolgt (außer der Undo-/Redo-Navigation selbst).
- **FR-008**: System MUSS die Undo- und Redo-Verläufe an die Anwendungssitzung binden: beim Schließen der Anwendung werden beide Verläufe entfernt; beim Neustart beginnen sie leer. Es wird nichts in die Datei oder anderweitig persistiert.
- **FR-009**: System MUSS Undo und Redo ausschließlich im normalen Bearbeitungsmodus wirken lassen. In Popup-, Such- und Bestätigungsmodi (z. B. ungespeicherte-Änderungen-Popup, Such-Eingabe, Konfliktabfrage) haben Ctrl-Z und Ctrl-Y keine Wirkung.
- **FR-010**: Jeder Undo-Schritt MUSS den Textzustand exakt in den Zustand unmittelbar vor der jeweiligen Änderung zurückversetzen; jeder Redo-Schritt MUSS ihn exakt in den Zustand unmittelbar nach dieser Änderung zurückversetzen, einschließlich Position und Inhalt über Zeilengrenzen und Unicode hinweg.
- **FR-011**: System MUSS definieren, was als ein einzelner Undo-/Redo-Schritt gilt: Jeder einzelne Tastendruck (Zeichen einfügen, einzelnes Zeichen löschen, Enter/Zeilenumbruch) bildet genau einen eigenen Undo-/Redo-Schritt. Es findet keine Gruppierung aufeinanderfolgender Eingaben statt, sodass das Verhalten für den Nutzer vorhersehbar bleibt.
- **FR-012**: System MUSS garantieren, dass Hin- und Herschalten zwischen Undo und Redo deterministisch bleibt: wiederholtes Undo gefolgt von wiederholtem Redo führt ohne Verlust zum zuletzt geänderten Zustand zurück.
- **FR-013**: System MUSS sicherstellen, dass beim Speichern einer Datei der Undo-/Redo-Verlauf erhalten bleibt (Speichern ist selbst keine neu aufzuzeichnende Textänderung), damit ein Nutzer auch nach dem Speichern weiter rückgängig machen kann.
- **FR-014**: System MUSS definieren, wie automatische Tests die primären Abläufe und die relevanten Randfälle abdecken: Aufbau eines Verlaufs, wiederholtes Undo/Redo, Leeren des Redo-Verlaufs bei neuer Änderung, Leben-pro-Sitzung-Verhalten, Unicode/Mehrzeil-Änderungen und die Behandlung der Stack-Enden. Jede Ausprägung muss ohne Terminal-UI-Rendern über die vorhandene EditingSession-Schnittstelle prüfbar sein.

### Key Entities *(include if feature involves data)*

- **Undo-Eintrag**: Repräsentiert eine rückgängig zu machende Änderung. Wesentliche Eigenschaften: der Zustand oder die Differenz, die beschreibt, wie der Text vor dieser Änderung aussah, sowie die Art der Änderung (Einfügen/Löschen/Ersetzen). Eine konkrete Repräsentation (Vollzustand vs. Differenz) wird in der Planung festgelegt.
- **Redo-Eintrag**: Repräsentiert eine per Undo zurückgenommene Änderung, die wiederhergestellt werden kann. Entsteht ausschließlich durch Undo und wird bei der nächsten neuen Änderung verworfen.
- **Undo-Verlauf**: Geordnete Folge von Undo-Einträgen, jüngster oben; wächst mit jeder neuen Textänderung und schrumpft mit jedem Undo. Lebensdauer ist an die Sitzung gebunden.
- **Redo-Verlauf**: Geordnete Folge von Redo-Einträgen; wächst mit jedem Undo, schrumpft mit jedem Redo und wird bei jeder neuen Textänderung vollständig geleert. Lebensdauer ist an die Sitzung gebunden.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Nutzer können nach beliebig vielen aufeinanderfolgenden Textänderungen durch wiederholtes Ctrl-Z den exakten Ausgangszustand des Dokuments (Zeichen für Zeichen, inklusive Unicode und Zeilenumbrüchen) wiederherstellen.
- **SC-002**: Nutzer können nach beliebig vielen Undo-Schritten durch wiederholtes Ctrl-Y den zuletzt geänderten Textzustand exakt wiederherstellen, ohne Verlust einzelner Schritte.
- **SC-003**: Nach einer neuen Textänderung sind zuvor per Undo zurückgenommene Schritte mit Ctrl-Y nicht mehr erreichbar, was durch einen Testfall nachgewiesen wird, der Redo nach einer Neu-Eingabe als wirkungslos bestätigt.
- **SC-004**: Nach dem Schließen und erneuten Öffnen der Anwendung ist der Verlauf leer; Ctrl-Z und Ctrl-Y bleiben beim frisch geöffneten Dokument ohne Wirkung, bis neue Änderungen vorgenommen werden.
- **SC-005**: Automatisierte Tests über die EditingSession-Schnittstelle decken die primäre Undo/Redo-Reise, das Leeren des Redo-Verlaufs bei neuer Änderung, das Sitzungs-Lebensdauer-Verhalten sowie Unicode- und Zeilenumbruch-bezogene Änderungen und die Stack-Enden ab.
- **SC-006**: Der Verlauf wächst mit jeder Änderung weiter, ohne dass eine fest codierte Obergrenze die Funktionalität vorzeitig beendet (nur der tatsächlich verfügbare Speicher begrenzt die Länge).
- **SC-007**: Im Grenzfall der Speichererschöpfung beim Aufnehmen eines neuen Schritts wird der älteste Undo-Schritt verworfen, um Platz zu schaffen; der Dokumenttext bleibt unbeschädigt, die Änderung wird ausgeführt, und der Nutzer wird informiert – nachgewiesen durch einen Testfall, der diesen Ablauf über die EditingSession-Schnittstelle prüft.

## Clarifications

### Session 2026-07-02

- Q: Wie soll sich der Editor bei Speichererschöpfung beim Aufnehmen eines neuen Undo-Schritts verhalten? → A: Ältesten Undo-Schritt verwerfen, Platz schaffen, Änderung anwenden und Nutzer informieren.

- Q: Wie soll die Schritt-Granularität für Undo/Redo sein (jeder Tastendruck vs. Gruppierung)? → A: Jeder Tastendruck ist ein eigener Schritt, keine Gruppierung.

## Assumptions

- **Schritt-Granularität**: Jeder einzelne Tastendruck (Zeichen einfügen, einzelnes Zeichen löschen, Enter/Zeilenumbruch) bildet genau einen eigenen Undo-/Redo-Schritt. Es erfolgt keine Gruppierung aufeinanderfolgender Eingaben; diese Festlegung ist verbindlich und nicht mehr der Planung überlassen.
- **Repräsentation**: Ob jeder Verlaufsschritt den Vollzustand oder eine Differenz speichert, wird in der Planung entschieden; die Spezifikation verlangt nur, dass Undo/Redo deterministisch und exakt arbeiten.
- **Geltungsbereich**: Undo/Redo betrifft ausschließlich Textänderungen am Dokumentpuffer. Navigation, Suchstatus und Moduswechsel erzeugen keine Schritte und sind nicht rückgängig zu machen.
- **Tastenbelegung**: Ctrl-Z steht für Undo, Ctrl-Y für Redo; bestehende Belegungen (Ctrl-S, Ctrl-Q, Ctrl-F, Ctrl-G) bleiben unberührt.
- **Speichern**: Speichern (`Ctrl-S`) zählt nicht als Textänderung und erzeugt keinen Undo-Schritt; der Verlauf bleibt beim Speichern erhalten.
- **Sitzungs-Bindung**: Die Verläufe leben im Arbeitsspeicher der Sitzung. Eine Persistenz über die Datei oder in Begleitdateien ist ausdrücklich nicht Teil dieses Features.
- **Existierende Architektur**: Das Feature wird konsistent in die bestehende, dokumentierte Modulstruktur (Editor-Status, Eingabe-Mapping, Rendering) eingebettet, ohne die bestehenden Single-Document-/Single-Binary-Grenzen zu überschreiten.
