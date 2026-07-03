# Feature Specification: Clipboard Cut, Copy & Paste

**Feature Branch**: `009-clipboard-cut-copy-paste`

**Created**: 2026-07-03

**Status**: Draft

**Input**: User description: "Cut, Copy und Paste Als Benutzer möchte ich, dass die Shortcuts Ctrl-X, Ctrl-C und Ctrl-V so funktionieren wie gewohnt. Sie sollen das System Clipboard benutzen. Wenn es eine Selektion gibt, beziehen sich X und C auf den Text der Selektion und bei V wird der gesamte selektierte Text durch den Inhalt des Clipboard ersetzt. Gibt es keine Selektion, so wirken X und C auf das Zeichen unter dem Cursor und V fügt den Text so ein, wie sonst ein normales Zeichen eingefügt wird. Ctrl-X und Ctrl-V sind atomare Aktionen bzgl. Undo. Das Clipboard wird aber nicht zurückgesetzt. Wenn also mit Ctrl-X ein Zeichen oder ein Text in die Zwischenablage gestellt und im Text selbst gelöscht wird, wird nur da Löschen mit Undo rückgängig gemacht, das Clipboard bleibt unverändert."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Kopieren (Copy, Ctrl-C) mit aktiver Selektion (Priority: P1)

Ein Nutzer hat einen Textbereich markiert und möchte ihn an eine andere Stelle in einem anderen Dokument oder einer anderen Anwendung kopieren. Durch Drücken von Ctrl-C wird der selektierte Text in das System-Clipboard übernommen und verbleibt im Editor am ursprünglichen Ort unverändert. Der Nutzer kann denselben Kopiervorgang beliebig oft wiederholen (z. B. um mehrere Ziele zu versorgen); jeder Vorgang überschreibt den Clipboard-Inhalt mit dem jeweils aktuell markierten Bereich.

**Why this priority**: Copy ist die Grundlage für das Wiederverwenden von Text ohne Datenverlust. Ohne diese Funktion ist manuelles Kopieren (Markieren, Ausschneiden, manuelles Einfügen, Rückgängigmachen) der einzige Weg — ineffizient und fehleranfällig.

**Independent Test**: Einen Textbereich markieren, Ctrl-C drücken, die Selektion beibehalten, und prüfen, dass der Clipboard-Inhalt dem markierten Bereich entspricht. Erwartete Wirkung: Text bleibt im Editor unverändert, System-Clipboard enthält den korrekten Inhalt.

**Acceptance Scenarios**:

1. **Given** der Text "Hallo Welt" mit selektiertem Teil "Welt", **When** der Nutzer Ctrl-C drückt, **Then** steht im System-Clipboard der Text "Welt", die Selektion bleibt bestehen, und der Editor-Inhalt verändert sich nicht.
2. **Given** dieselbe Ausgangslage und ein externes Fenster, **When** der Nutzer in dieses Fenster Cmd+V (bzw. Ctrl+V unter Linux/Windows) drückt, **Then** wird "Welt" an der Cursorposition des externen Fensters eingefügt.
3. **Given** der Text "Hallo Welt" mit selektiertem Teil "Hallo", **When** der Nutzer erneut Ctrl-C drückt, **Then** aktualisiert sich das System-Clipboard auf "Hallo"; vorheriger Inhalt ("Welt") ist ersetzt.

---

### User Story 2 - Ausschneiden (Cut, Ctrl-X) ohne Selektion (Priority: P1)

Ein Nutzer möchte ein einzelnes Zeichen an einer bestimmten Cursorposition entfernen und in einem anderen Kontext wiederverwenden. Der Cursor steht zwischen zwei Zeichen (oder am Anfang/Ende des Dokuments). Durch Drücken von Ctrl-X wird das rechts vom Cursor stehende Zeichen (bzw. das letzte Zeichen bei Position am Dokumentende) in das System-Clipboard übernommen und im Editor gelöscht. Dieser Vorgang ist atomar: ein einzelnes Undo stellt sowohl den gelöschten Text als auch die verschobenen Nachbarzeichen wieder her.

**Why this priority**: Einzige Zeichen ausschneiden ist eine effiziente Alternative zu Markieren+Entfernen, insbesondere bei mehrzeiligen Dokumenten oder schnellen Bearbeitungen.

**Independent Test**: Cursor zwischen zwei Zeichen positionieren (oder am Dokumentanfang/-ende), Ctrl-X drücken, Editor-Inhalt prüfen und Undo auslösen. Erwartete Wirkung: Zeichen verschwindet mit einem Undo vollständig in der ursprünglichen Position.

**Acceptance Scenarios**:

1. **Given** der Text "HalloWelt" mit Cursor zwischen "o" und "W", **When** der Nutzer Ctrl-X drückt, **Then** steht im System-Clipboard "W" und im Editor nur noch "Halloelt".
2. **Given** dieselbe Ausgangslage (Editor enthält nun "Halloelt"), **When** der Nutzer Ctrl-Z drückt, **Then** wird der Text sofort wieder zu "HalloWelt"; der Undo-Pfad stellt sowohl das Einfügen von "W" als auch das Löschen dar; das System-Clipboard bleibt unverändert auf "W".
3. **Given** der Cursor steht am Ende des Dokuments in einem Dokument mit Inhalt "Text\n" (ein abschließendes Newline), **When** der Nutzer Ctrl-X drückt, **Then** wird das letzte Zeichen ("\n") in das Clipboard kopiert und bleibt nicht im Text erhalten; nach Undo steht das Newline wieder an seiner ursprünglichen Position.
4. **Given** es existieren bereits Einträge im System-Clipboard aus früheren Copy/Cut-Aktionen (z. B. "TextA"), **When** der Nutzer Ctrl-X für ein einzelnes Zeichen (z. B. "Z") auslöst, **Then** überschreibt der Cut-Vorgang das Clipboard mit "Z"; vorheriger Inhalt ("TextA") ist ersetzt.

---

### User Story 3 - Ausschneiden (Cut, Ctrl-X) mit aktiver Selektion (Priority: P1)

Ein Nutzer hat einen mehrzeiligen Textbereich markiert und möchte ihn an eine andere Stelle verschieben. Durch Drücken von Ctrl-X wird der gesamte selektierte Textinhalt in das System-Clipboard übernommen und im Editor entfernt; die verbleibenden Textteile rücken zusammen (z. B. Zeilen verschmelzen). Dieser Vorgang ist atomar: ein einzelnes Undo stellt den gesamten gelöschten Bereich inklusive aller Zeilenumbrüche wieder her.

**Why this priority**: Das Verschieben von Textblöcken ist eine der häufigsten Bearbeitungs-Aktionen; ohne Cut müsste der Nutzer manuell löschen, zur Zielposition springen und einfügen — fehleranfällig bei langen Dokumenten.

**Independent Test**: Einen mehrzeiligen Bereich markieren, Ctrl-X drücken, den neuen Editor-Inhalt prüfen und Undo auslösen. Erwartete Wirkung: Text verschwindet mit einem Undo vollständig in der ursprünglichen Position (inklusive Zeilenumbrüchen).

**Acceptance Scenarios**:

1. **Given** einen Text über drei Zeilen ("Zeile1\nZeile2\nZeile3\n") mit selektiertem "Zeile2\n" (die komplette zweite Zeile inklusive Newline), **When** der Nutzer Ctrl-X drückt, **Then** steht im System-Clipboard "Zeile2\n" und im Editor nur noch "Zeile1\nZeile3\n"; die dritte Zeile rückt an die Position der zweiten.
2. **Given** dieselbe Ausgangslage (Editor enthält nun "Zeile1\nZeile3\n"), **When** der Nutzer Ctrl-Z drückt, **Then** wird "Zeile2\n" vollständig wiederhergestellt; der Undo-Punkt stellt den Zustand vor dem Cut her.
3. **Given** eine Selektion über mehrere Zeichen einer einzelnen Zeile ("Hallo Welt", selektiert "lo W"), **When** der Nutzer Ctrl-X drückt, **Then** steht im System-Clipboard "lo W" und im Editor nur noch "Halt Welt".
4. **Given** der Textinhalt ist identisch mit dem aktuellen Clipboard-Inhalt (der Nutzer hat zuvor diesen exakten Bereich kopiert), **When** der Nutzer den Originalbereich löscht und die Paste-Funktion auslöst, **Then** steht der ursprünglich gelöschte Text wieder an derselben Stelle; der Undo-Pfad besteht nur aus der Rückgängig-Machung des Paste-Vorgangs (der Clipboard-Inhalt bleibt erhalten).

---

### User Story 4 - Einfügen (Paste, Ctrl-V) ohne Selektion (Priority: P1)

Ein Nutzer hat zuvor mit Copy oder Cut Text in das System-Clipboard übernommen und möchte ihn an einer beliebigen Cursorposition im aktuellen Dokument einfügen. Durch Drücken von Ctrl-V wird der Clipboard-Inhalt genau an dieser Cursorposition eingefügt; die Zeichen rechts vom Cursor rücken nach rechts (bzw. Zeilen darunter werden bei mehrzeiligem Inhalt entsprechend erweitert). Dieser Vorgang ist atomar: ein einzelnes Undo entfernt den eingefügten Text und stellt die vorherige Dokumentstruktur vollständig wieder her.

**Why this priority**: Einfügen ist der komplementäre Schritt zu Kopieren/Ausschneiden — ohne ihn bleiben übernommene Inhalte im Clipboard nutzlos.

**Independent Test**: Zuerst einen Textbereich kopieren, dann an einer anderen Stelle im Dokument Ctrl-V auslösen und prüfen, dass der Clipboard-Inhalt korrekt eingefügt wurde. Undo wiederherstellen und prüfen. Erwartete Wirkung: Inhalt erscheint an der Cursorposition; Undo stellt den Zustand davor her.

**Acceptance Scenarios**:

1. **Given** System-Clipboard enthält "Test" durch eine frühere Copy-Aktion des Nutzers, der Editor enthält "HalloWelt" mit Cursor zwischen "o" und "W", **When** der Nutzer Ctrl-V drückt, **Then** steht im Editor "HalloTestWelt" und der Cursor zeigt direkt hinter dem eingefügten "T".
2. **Given** dieselbe Ausgangslage (Editor enthält "HalloTestWelt"), **When** der Nutzer Ctrl-Z drückt, **Then** wird der ursprüngliche Text "HalloWelt" wiederhergestellt; das eingefügte Test verschwindet vollständig mit einem Undo-Schritt; der Clipboard-Inhalt bleibt unverändert weiter auf "Test".
3. **Given** System-Clipboard enthält einen mehrzeiligen Text ("Zeile1\nZeile2") durch eine frühere Copy-Aktion, der Editor enthält "StartEnde" mit Cursor zwischen "t" und "E", **When** der Nutzer Ctrl-V drückt, **Then** steht im Editor:
   ```
   Start
   Zeile1
   Zeile2
   Ende
   ```
4. **Given** System-Clipboard ist leer (z. B. aufgrund eines früheren Clear-Befehls oder neuem Editor-Start ohne Kopieraktion), **When** der Nutzer Ctrl-V drückt, **Then** wird nichts eingefügt; die Aktion hat keine sichtbare Wirkung auf den Editor-Inhalt und erzeugt keinen Undo-Eintrag.

---

### User Story 5 - Einfügen (Paste, Ctrl-V) mit aktiver Selektion (Priority: P2)

Ein Nutzer hat einen Textbereich markiert und möchte ihn durch den Clipboard-Inhalt ersetzen (überschreiben). Durch Drücken von Ctrl-V wird der gesamte selektierte Bereich im Editor gelöscht und der Clipboard-Inhalt an seiner Stelle eingefügt. Dieser Ersatzvorgang ist atomar: ein einzelnes Undo stellt sowohl den vorherigen selektierten Text als auch die umgebenden Textteile wieder her.

**Why this priority**: Ersetzen durch Einfügen erlaubt gezielte Textauswahl — der Nutzer kann einen fehlerhaften Bereich markieren und ihn direkt mit dem richtigen Clipboard-Inhalt überschreiben, ohne separat löschen und dann einfügen zu müssen.

**Independent Test**: Einen Bereich markieren, Clipboard enthält "Korrektur", Ctrl-V auslösen, prüfen dass Markierung durch Clipboard-Inhalt ersetzt wurde. Undo wiederherstellen. Erwartete Wirkung: selektierter Bereich wird vollständig durch Clipboard-Inhalt ersetzt; Undo stellt Originaltext her.

**Acceptance Scenarios**:

1. **Given** der Text "Hallo AlteWelt" mit selektiertem Teil "Alte", System-Clipboard enthält "Neue", **When** der Nutzer Ctrl-V drückt, **Then** steht im Editor "Hallo NeueWelt"; der gesamte markierte Bereich ist durch den Clipboard-Inhalt ersetzt.
2. **Given** dieselbe Ausgangslage (Editor enthält "Hallo NeueWelt" nach Paste über Selektion), **When** der Nutzer Ctrl-Z drückt, **Then** wird "Hallo AlteWelt" vollständig wiederherstellt.
3. **Given** eine Selektion, die mehrere Zeilen umfasst ("Zeile1\nZeile2", beides selektiert), System-Clipboard enthält "Ersatz", **When** der Nutzer Ctrl-V drückt, **Then** werden beide ursprünglichen Zeilen durch den Clipboard-Inhalt ersetzt; die umgebenden Zeilen des Dokuments bleiben unverändert.
4. **Given** es existieren bereits Einträge im System-Clipboard aus früheren Copy/Cut-Aktionen (z. B. "TextA"), **When** der Nutzer eine Paste-Operation über eine Selektion auslöst, **Then** ändert sich das Clipboard nicht; nachfolgende Paste-Vorgänge verwenden weiterhin denselben Inhalt ("TextA").

---

### Edge Cases

- Was passiert, wenn das System-Clipboard keinen Textinhalt sondern nur Binärdaten oder einen leeren String enthält? Der Editor MUSS nur textuellen Clipboard-Inhalt verarbeiten; binäre oder nicht-Text-Inhalte werden ignoriert und führen zu keiner sichtbaren Einfüge-Aktion.
- Bei einem sehr großen Clipboard-Inhalt (> 1 MB): Der Editor MUSS Inhalte über 1 MB ablehnen und den Nutzer informieren; absturz- und hängerfrei bleiben.
- Was passiert, wenn parallel zum Cut ein anderer Prozess den Clipboard-Inhalt überschreibt? Das System Clipboard wird durch die nächste Copy/Cut-Aktion überschrieben; der Editor verwaltet keinen eigenen "Zustand" des Clipboards – dies ist eine reine System-API-Funktion. Der im Editor durchgeführten Undo-Pfad referenziert immer den Original-Inhalt zum Zeitpunkt der Cut/Aktion.
- Wie verhält sich Paste, das selbst auch einen Cut enthält? Wenn der Clipboard-Inhalt "ABC" ist und der Nutzer Ctrl-V auslöst, wird "ABC" eingefügt. Falls zwischen dem ursprünglichen Copy/Cut und der Paste ein anderer Prozess den Clipboard-Inhalt ändert, wird der *aktuelle* Clipboard-Inhalt eingefügt.
- Was passiert bei Undo nach einem Cut? Der Undo-Pfad stellt nur den im Text gelöschten Bereich wieder her, aber das System-Clipboard behält den Inhalt, der beim Cut gesetzt wurde (wie vom Nutzer beschrieben). Der Nutzer kann erneut Ctrl-V drücken und damit denselben Inhalt erneut einfügen — unabhängig von Undo.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Das Drücken von Ctrl-C (Copy) MUSS den aktuell selektierten Text in das System-Clipboard kopieren und die Selektion im Editor beibehalten; wenn keine Selektion existiert, MUSS es das Zeichen unter dem Cursor (erstes Graphem-Cluster rechts vom Cursor) in das Clipboard kopieren **ohne** den Editor-Inhalt zu verändern. Das Zeichen bleibt am Originalort erhalten.
- **FR-002**: Das Drücken von Ctrl-X (Cut) MUSS bei bestehender Selektion den selektierten Text in das System-Clipboard kopieren und ihn im Editor löschen; die verbleibenden Textteile rücken zusammen (einschließlich möglicher Zeilenumbrüche).
- **FR-003**: Das Drücken von Ctrl-X (Cut) MUSS bei fehlender Selektion das Zeichen rechts vom Cursor in das System-Clipboard kopieren und im Editor löschen; am Dokumentende soll das letzte Zeichen gelöscht werden.
- **FR-004**: Das Drücken von Ctrl-V (Paste) MUSS den aktuellen Inhalt des System-Clipboards an der Cursorposition einfügen; bei bestehender Selektion MUSS vorher der selektierte Bereich vollständig durch den Clipboard-Inhalt ersetzt werden.
- **FR-005**: Alle Cut- und Paste-Vorgänge (Ctrl-X, Ctrl-V) MÜSSEN als einzelne atomare Einheit im Undo-Verlauf aufgezeichnet werden; ein einzelnes Ctrl-Z stellt den Zustand vor der Aktion vollständig wieder her.
- **FR-006**: Ein Cut-Vorgang überschreibt den bisherigen Inhalt des System-Clipboards mit dem neuen Text; er MUSS den bestehenden Clipboard-Inhalt unverändert lassen, sobald der Cut abgeschlossen ist.
- **FR-007**: Ein Paste-Vorgang MUSS die Selektion (falls vorhanden) nach dem Einfügen aufheben und den Cursor hinter dem eingefügten Text positionieren.
- **FR-008**: Der Editor darf keine eigenen Kopien des System-Clipboards verwalten; er MUSS ausschließlich die vom Betriebssystem bereitgestellte Clipboard-API verwenden, um Lese- und Schreibzugriffe zu gewährleisten.
- **FR-009**: Falls das System-Clipboard leeren oder nicht-textuellen Inhalt enthält, MUSS der Editor bei Ctrl-V eine stille No-Op ausführen (keine sichtbare Wirkung, kein Undo-Eintrag).
- **FR-010**: Bei einem Cut mit bestehender Selektion MUSS die atomare Undo-Einheit den gesamten entfernten Bereich exakt wiederherstellen — einschließlich aller Zeilenumbrüche und Spezialzeichen — an der ursprünglichen Position.
- **FR-011**: Der Cut des einzelnen Zeichens (ohne Selektion) MUSS dasselbe Verhalten wie FR-002 haben — in einer atomaren Undo-Einheit das Zeichen rechts vom Cursor löschen — mit gleichem Cursor-Endzustand (Cursor steht an derselben Stelle, an der das Zeichen eingefügt wurde).
- **FR-012**: Der Editor MUSS sicherstellen, dass keine Datenverluste auftreten, wenn ein Cut-Vorgang während einer Clipboard-Aktion durch einen anderen Prozess unterbrochen wird oder umgekehrt; der Undo-Pfad MUSS immer konsistent bleiben.
- **FR-013**: Alle Copy/Cut/Paste-Vorgänge MÜSSEN Clipboard-Inhalte über 1 MB ablehnen. Größere Inhalte werden zurückgewiesen, der Editor bleibt korrekt funktionsfähig, und der Nutzer erhält eine nicht-blockierende Statusmeldung.

### Key Entities *(include if feature involves data)*

- **SystemClipboard**: Ein externes, vom Betriebssystem bereitgestellter Zwischenspeicher für Textdaten. Der Editor fragt diesen zur Laufzeit ab und schreibt nur Textinhalte hinein; die Lebensdauer des Inhalts liegt außerhalb der Kontrolle des Editors (kann von anderen Prozessen überschrieben werden).
- **CutState** (intern): Repräsentiert einen ausgelösten Cut-Vorgang. Besteht aus dem entfernten Textfragment (das auch ins Clipboard geschrieben), der ursprünglichen Einfügemarke und einer Undo-Eintrag. Beziehung: wird durch `Clipboard` und `History` verwaltet; die Atomarität ist im Undo garantiert, nicht im System-Clipboard.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Nutzer können mit Ctrl-C einen markierten Textbereich in 100 % der Fälle unverändert im Editor kopieren und zur Verwendung im Clipboard sicherstellen; die Selektion bleibt visuell erhalten.
- **SC-002**: Mit Ctrl-X kann ein Nutzer einen selektierten Bereich oder ein einzelnes Zeichen (ohne Selektion) in jedem Fall vollständig löschen; Undo stellt den gesamten Bereich in genau 100 % der Fälle mit einem einzigen Undo-Schritt wieder her.
- **SC-003**: Ein einziger Paste-Vorgang mit Ctrl-V setzt den System-Clipboard-Inhalt an genau der Cursorposition ein (oder ersetzt die Selektion); Undo entfernt alles in einem Schritt, ohne Resteffekte.
- **SC-004**: Das System-Clipboard wird nach jedem Cut oder Copy aktualisiert; nachfolgende Einfügeoperationen verwenden den aktuellsten Inhalt; frühere Clipboard-Einträge bleiben bestehen, solange sie nicht überschrieben werden.
- **SC-005**: Der Editor MUSS keine Abstürze, Hänger oder Datenverluste verursachen, wenn das System-Clipboard leer ist, binäre Inhalte liefert oder einen sehr großen Inhalt (> 1 MB) enthält. Dies gilt explizit für die in FR-013 und der Edge-Case-Liste genannten Fälle.

## Clarifications

### Session 2026-07-03

- Q: Was soll Ctrl-C ohne aktive Selektion tun? — A: Option A — kopiert das Zeichen unter dem Cursor (gleiche Logik wie Cut), um Symmetrie mit Ctrl-X zu gewährleisten.

FR-001 wurde aktualisiert, um dieses Verhalten zu kodifizieren.

- Q: Welche Obergrenze für Clipboard-Inhalt?
A: Option A — harte Grenze von 1 MB.

## Assumptions

- Das Clipboard ist ein textbasiertes System-Dienstprogramm ohne mehrformatige Unterstützung (z. B. HTML vs. Plaintext); der Editor speichert nur den einfachsten Textinhalt ab und stellt ihn als solchen wieder her.
- Die bestehende Undo/Redo-Architektur des Editors unterstützt bereits atomare EditSteps; die Cut-/Paste-Aktionen erweitern diese Architektur nicht grundlegend sondern nutzen sie wie normale Textoperationen (Löschen bzw. Einfügen).
- System Clipboard ist ein Singleton (ein Prozess zur Zeit); mehrere parallele Clipboard-Zugriffe vom selben oder anderen Prozessen werden durch das OS serialisiert.
- Der Begriff "Zeichen unter dem Cursor" bezieht sich auf das erste vollständige Graphem-cluster am Cursor; dasCut-Konzept MUSS die korrekte Handhabung von Multi-Cluster-Zeichen sowie CRLF gewährleisten (kein halbierter Cluster).
