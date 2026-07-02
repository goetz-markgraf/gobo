# Feature Specification: Text Selection

**Feature Branch**: `007-text-selection`

**Created**: 2026-07-02

**Status**: Draft

**Input**: User description: "Selektion Als User möchte ich mit Shift und Cursor-Bewegungen Text markieren. Aktionen, während eine Selektion existiert, sollen so laufen, wie man es erwartet: - Eine Eingabe löscht den selektierten Text und ersetzt ihn -> Undo beachten. Das Ersetzen ist eine atomare Aktion (Löschen und Schreiben in einem Schritt. Wenn es einfacher ist, das in zwei Schritten zu tun - 1. Löschen, 2. Schreiben -, ist das auch OK - Cursor-Bewegungen ohne Shift nehmen die Selektion wieder zurück - DEL oder BACKSPACE löscht den markieren Text -> Undo beachten, atomare Aktion"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Text markieren (Priority: P1)

Ein Nutzer hält Shift gedrückt und bewegt den Cursor mit den Pfeiltasten über den Text. Dabei wird der Bereich zwischen der Startposition und der aktuellen Cursorposition als Selektion hervorgehoben. Die Selektion wächst oder schrumpft, während der Nutzer die Cursor-Bewegung mit gehaltener Shift-Taste fortsetzt, und kann sowohl vorwärts als auch rückwärts (gegen die Textrichtung) aufgebaut werden.

**Why this priority**: Die Selektion ist die Grundlage für alle weiteren Stories. Ohne sichtbare, navigierbare Markierung gibt es keinen Aktionskontext für Ersetzen oder Löschen. Diese Reise liefert für sich allein einen nutzbaren Mehrwert: der Nutzer kann einen Bereich visuell erfassen.

**Independent Test**: Kann vollständig getestet werden, indem ein Dokument mit bekanntem Text geöffnet, Shift+Pfeiltasten-Sequenzen gesendet und der resultierende Selektionsbereich (Start und Ende sowie Richtung) sowie die Hervorhebung geprüft werden. Erwartete Wirkung: Die Selektion entspricht nach jeder Bewegung genau dem Bereich zwischen Anker und aktuursor.

**Acceptance Scenarios**:

1. **Given** ein offenes Dokument mit Text "Hallo", Cursor hinter dem letzten Zeichen, **When** der Nutzer Shift+Left dreimal drückt, **Then** sind die Zeichen "llo" selektiert und der Cursor steht am Anfang der Selektion.
2. **Given** ein Dokument mit Text "Hallo", Cursor am Zeilenanfang, **When** der Nutzer Shift+Right zweimal drückt, **Then** sind die Zeichen "Ha" selektiert und der Cursor steht am Ende der Selektion.
3. **Given** eine bestehende Selektion von 3 Zeichen, **When** der Nutzer mit gehaltener Shift-Taste in entgegengesetzte Richtung bewegt, **Then** schrumpft die Selektion entsprechend und kann bis auf null zurückgehen, ohne darüber hinaus negativ zu wachsen.
4. **Given** eine bestehende Selektion, **When** der Nutzer Shift+Up bzw. Shift+Down drückt, **Then** wird die Selektion auf die jeweilige Zeile ober- bzw. unterhalb erweitert und umfasst den Bereich zwischen Anker und neuer Cursorposition.

---

### User Story 2 - Selektion zurücknehmen (Priority: P2)

Ein Nutzer hat eine Selektion aufgebaut und bewegt den Cursor mit einer Pfeiltaste ohne gehaltene Shift-Taste. Die Selektion wird sofort aufgehoben; der Cursor springt an das Ende der Bewegung (bzw. an die Position gemäß der üblichen Bewegungsregel), und es ist kein Textbereich mehr hervorgehoben.

**Why this priority**: Cursor-Bewegung ohne Shift ist die natürliche Methode, eine Markierung zu verlassen. Ohne dieses Verhalten bliebe eine versehentlich aufgebaute Selektion dauerhaft aktiv und blockiert normalen Textfluss.

**Independent Test**: Kann getestet werden, indem eine Selektion aufgebaut und dann eine Pfeiltaste ohne Shift gesendet wird. Erwartete Wirkung: die Selektion ist leer, der Cursor steht an einer definierten Position, weiterer Text ist nicht hervorgehoben.

**Acceptance Scenarios**:

1. **Given** eine bestehende Selektion von "llo" im Text "Hallo", Cursor am Anfang der Selektion, **When** der Nutzer Right ohne Shift drückt, **Then** ist die Selektion aufgehoben und der Cursor steht eine Position weiter rechts.
2. **Given** eine bestehende Selektion, **When** der Nutzer Left ohne Shift drückt, **Then** ist die Selektion aufgehoben und der Cursor steht an der dem Anfang der Bewegung entsprechenden Position.
3. **Given** eine bestehende Selektion über mehrere Zeilen, **When** der Nutzer Up oder Down ohne Shift drückt, **Then** ist die Selektion aufgehoben und der Cursor bewegt sich in die jeweilige Nachbarzeile unter Beibehaltung der bevorzugten Sichtspalte.

---

### User Story 3 - Selektion durch Eingabe ersetzen (Priority: P3)

Ein Nutzer hat eine Selektion und tippt ein Zeichen. Der selektierte Text wird entfernt und das getippte Zeichen an dessen Stelle eingefügt, sodass der Text inline weiterfließt. Dieser Vorgang ist als atomare Aktion in den Undo-Verlauf eingetragen: ein einzelner Ctrl-Z-Schritt stellt den gelöschten Text wieder her und entfernt das neu eingetippte Zeichen.

**Why this priority**: Ersetzen durch Eingabe ist die am häufigsten genutzte Aktion auf einer Selektion. Sie schließt den Kreis aus Markieren und Bearbeiten und macht die Selektion erst produktiv.

**Independent Test**: Kann getestet werden, indem ein Dokument mit bekanntem Text geöffnet, eine Selektion aufgebaut, ein Zeichen gesendet und anschließend Ctrl-Z ausgelöst wird. Erwartete Wirkung: nach der Eingabe steht das neue Zeichen anstelle der Selektion; nach Ctrl-Z ist der Originaltext samt Selektion wiederhergestellt (bzw. der Zustand vor dem Ersetzen).

**Acceptance Scenarios**:

1. **Given** der Text "Hallo" mit selektiertem "llo", **When** der Nutzer "x" eintippt, **Then** steht der Text "Hax" und es existiert keine Selektion mehr, der Cursor steht hinter dem "x".
2. **Given** die Ersetzung aus Szenario 1, **When** der Nutzer einmal Ctrl-Z drückt, **Then** ist der Zustand vor der Ersetzung wiederhergestellt (oder der Text "Hallo" mit wiederhergestelltem Originalbereich), und das "x" ist entfernt.
3. **Given** eine Selektion über mehrere Zeichen, **When** der Nutzer mehrere Zeichen eintippt, **Then** wird die gesamte Selektion in einer atomaren Undo-Einheit gelöscht und die getippten Zeichen gemeinsam eingefügt; Ctrl-Z macht den gesamten Vorgang in einem Schritt rückgängig.
4. **Given** eine Selektion der Länge null ist nicht zulässig; **When** keine Selektion existiert, **Then** verhält sich die Eingabe wie bisher (Einfügen am Cursor) und erzeugt den normalen Undo-Eintrag.

---

### User Story 4 - Selektion löschen (Priority: P4)

Ein Nutzer hat eine Selektion und drückt DEL oder BACKSPACE. Der selektierte Text wird entfernt und der Cursor rückt an die Position, an der die Selektion begann. Dieser Vorgang ist eine atomare Aktion im Undo-Verlauf: ein einzelner Ctrl-Z-Schritt stellt den gelöschten Text wieder her.

**Why this priority**: Löschen einer Selektion ist die zweite zentrale Bearbeitungsaktion. Sie ist Voraussetzung für typische Vorschaus-Szenarien wie "Markieren und Entfernen".

**Independent Test**: Kann getestet werden, indem eine Selektion aufgebaut und DEL bzw. BACKSPACE gesendet wird. Erwartete Wirkung: der selektierte Bereich ist entfernt, der Cursor steht an der Anfangsposition; Ctrl-Z stellt den Text wieder her.

**Acceptance Scenarios**:

1. **Given** der Text "Hallo" mit selektiertem "llo", **When** der Nutzer DEL drückt, **Then** steht der Text "Ha", die Selektion ist leer und der Cursor steht hinter dem "a".
2. **Given** dieselbe Ausgangslage, **When** der Nutzer BACKSPACE drückt, **Then** steht der Text "Ha", die Selektion ist leer und der Cursor steht hinter dem "a" (gleiche Wirkung wie DEL, da die Selektion den löschbaren Bereich bestimmt).
3. **Given** die Löschung aus Szenario 1, **When** der Nutzer Ctrl-Z drückt, **Then** ist der Text "Hallo" wiederhergestellt.
4. **Given** eine Selektion über mehrere Zeilen, **When** der Nutzer DEL drückt, **Then** werden alle betroffenen Zeilen inklusive der dazwischenliegenden Zeilenumbrüche entfernt und der Cursor steht am Selektionsanfang.
5. **Given** es existiert keine Selektion, **When** der Nutzer DEL bzw. BACKSPACE drückt, **Then** verhält sich die Aktion wie bisher (Zeichen rechts bzw. links vom Cursor löschen) und erzeugt den normalen Undo-Eintrag.

---

### Edge Cases

- Was passiert, wenn der Nutzer Shift+Cursor über den Anfang oder das Ende des Dokuments hinaus bewegt? Die Selektion darf nicht über die Dokumentgrenzen hinaus wachsen; der Cursor wird an der Grenze gehalten, die Selektion endet dort.
- Was passiert bei Shift+Cursor-Bewegung mit umgekehrter Richtung, sodass der Cursor über den Selektionsanker hinaus läuft? Die Selektionsrichtung kehrt sich um und der Ananker bleibt fix, die Selektion wächst in die neue Richtung.
- Was passiert, wenn der Cursor dieselbe Position wie der Anker einnimmt (Selektionslänge null)? Es gibt keine sichtbare Selektion; Eingaben und Löschungen verhalten sich wie ohne Selektion.
- Wie wird eine Selektion dargestellt, die nur aus Zeilenumbrüchen besteht oder eine leere Zeile vollständig umfasst? Sie wird konsistent als Bereich hervorgehoben und ein Löschvorgang entfernt genau die enthaltenen Umbrüche.
- Was passiert, wenn während einer Selektion ein Befehl außerhalb des Editiermodus (z. B. Ctrl-F Suche, Ctrl-S Speichern, Quit-Prompt) ausgelöst wird? Die Selektion darf nicht versehentlich gelöscht werden; Such- und Speicherbefehle beeinträchtigen die Selektion nicht, und der Cursorbewegungs- bzw. Editierkontext bleibt erhalten, außer der Moduswechsel verlangt explizit etwas anderes.
- Was verhindert versehentlichen Datenverlust, wenn eine Selektion mehrere Zeilen umfasst und mit einer einzelnen Eingabe oder DEL gelöscht wird? Die atomare Undo-Einheit garantiert, dass der gesamte Inhalt mit einem Ctrl-Z-Schritt zurückholbar ist.
- Wie verhält sich die Selektion in dieser Unicode- und Zeichenindex-basierten Codebase bei mehrzeiligen Graphem-Clustern oder CRLF-Zeilenenden? Die Selektion arbeitet konsistent auf Zeichenebenen und umschließt CRLF- bzw. Multi-Graphem-Inhalte korrekt, ohne halbierte Cluster zu erzeugen.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Der Editor MUST eine Aktivierung von Selektionsmodus durch Halten der Shift-Taste bei Cursor-Bewegungen ermöglichen (Left, Right, Up, Down), sodass ein Bereich zwischen einer festen Ankerposition und der aktuellen Cursorposition markiert wird.
- **FR-002**: Der Editor MUST die Selektionsrichtung (vorwärts oder rückwärts zum Dokumentende) korrekt abbilden und beibehalten, sodass aufeinanderfolgende Shift-Bewegungen die Selektion konsistent wachsen oder schrumpfen lassen, auch über den Anker hinaus.
- **FR-003**: Die Selektion MAY nicht über die Dokumentgrenzen (Anfang bzw. Ende) hinaus wachsen; der Cursor wird an der Grenze gehalten und die Selektion endet dort.
- **FR-004**: Cursor-Bewegungen ohne gehaltene Shift-Taste MUST die bestehende Selektion sofort aufheben und den Cursor gemäß der jeweiligen Bewegungsregel neu platzieren.
- **FR-005**: Eine beliebige Zeicheneingabe, während eine nicht-leere Selektion existiert, MUST den gesamten selektierten Text in einer atomaren Undo-Aktion entfernen und das eingegebene Zeichen an der Anfangsposition der Selektion einfügen; danach existiert keine Selektion mehr und der Cursor steht hinter dem eingefügten Zeichen.
- **FR-006**: Drücken von DEL oder BACKSPACE, während eine nicht-leere Selektion existiert, MUST den gesamten selektierten Text in einer atomaren Undo-Aktion entfernen und den Cursor an die Anfangsposition der Selektion setzen; DEL und BACKSPACE haben dabei gleiche Wirkung.
- **FR-007**: Die atomaren Aktionen aus FR-005 und FR-006 (Löschen mit optionalem nachfolgendem Einfügen) MUST als einzelner Schritt im Undo-Verlauf aufgezeichnet werden, sodass genau ein Ctrl-Z den Zustand vor der Aktion wiederherstellt; ein nachfolgendes Ctrl-Y (Redo) stellt die Aktion erneut her.
- **FR-008**: Liegt keine Selektion vor (Selektion leer), so MUST die Eingabe sowie DEL/BACKSPACE sich unverändert zum bestehenden Verhalten (Einfügen bzw. Löschen Einzelzeichen / atomarer bestehender Eintrag) verhalten.
- **FR-009**: Befindet sich die Selektion über mehrere Zeilen, so MUST ein Ersetz- oder Löschvorgang alle betroffenen Zeilen einschließlich der dazwischenliegenden Zeilenumbrüche in der atomaren Aktion umfassen.
- **FR-010**: Der Visualisierungs-/Render-Pfad MUST den selektierten Bereich für den Nutzer sichtbar hervorheben (z. B. invers oder farbig), sodass der markierte Text eindeutig erkennbar ist; außerhalb der Selektion bleibt die normale Darstellung erhalten.
- **FR-011**: Nicht editierende Befehle (z. B. Suche, Speichern, Quit-Prompt) MAY die Selektion nicht unbeabsichtigt verändern; Modswechsel dürfen die Selektion nur dann verwerfen, wenn es der Moduswechsel ausdrücklich erfordert.
- **FR-012**: System MUST definiertes, sicheres Verhalten aufweisen, wenn Selektionen an Dokument- und Zeilengrenzen, bei leeren Zeilen, bei rein aus Zeilenumbrüchen bestehenden Bereichen sowie bei mehrzeiligen Unicode-Graphem-Clustern und CRLF-Zeilenenden auftreten, ohne halbierte Cluster zu erzeugen oder字符grenzen zu verletzen.
- **FR-013**: System MUST definierte, klare Grenzen zwischen den Verantwortlichkeiten einhalten (Cursor/Selektionszustand, Textpuffer-Mutation, Undo-Verlauf, Rendering, Eingabemapping), sodass die Selektionslogik nicht implizit Zustand anderer Module verwaltet oder kontrolliert.
- **FR-014**: System MUST automatisierte Tests vorsehen, die den primären Selektionsfluss (Aufbau, Schrumpfen, Umkehren der Richtung), das Zurücknehmen ohne Shift, das Ersetzen durch Eingabe sowie das Löschen mit DEL/BACKSPACE abdecken, jeweils einschließlich Undo-Rundlauf, Mehrzeiligkeit und der definierten Grenzfälle.

### Key Entities *(include if feature involves data)*

- **Selection**: Stellt einen markierten Textbereich dar. Schlüsselattribute: fester Ankerpunkt (Anchor) und aktuelle Cursorposition (Head), beide als Zeichenindizes im Dokument; implizite Richtung (vorwärts, wenn Head ≥ Anchor, sonst rückwärts). Beziehung: überlagert den `CursorState` und wird von der `EditingSession` gehalten; Leerbedingung (Anchor == Head) bedeutet keine Selektion.
- **EditStep (Selektionsvariante)**: Erweitert den bestehenden Undo-Verlaufseintrag um eine atomare Aktion, die einen entfernt/gelöschten Bereich und ein optional eingefügtes Fragment in einem Schritt kapselt. Beziehung: wird durch `history.record` erfasst und per Ctrl-Z / Ctrl-Y zurück- bzw. wiederaufgeführt.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Nutzer können mit Shift+Cursor-Bewegungen innerhalb weniger Sekunden einen sichtbar markierten Textbereich aufbauen, schrumpfen oder umkehren, und die Markierung entspricht zu jedem Zeitpunkt exakt dem Bereich zwischen Anker und Cursor.
- **SC-002**: Cursor-Bewegung ohne Shift führt dazu, dass in 100 % der Fälle eine bestehende Selektion sofort aufgehoben ist und der Cursor subsequent normal positioniert ist.
- **SC-003**: Eine Eingabe oder DEL/BACKSPACE auf eine Selektion ersetzt bzw. löscht den gesamten markierten Bereich in einer einzigen, atomaren Undo-Einheit, sodass Nutzer in jedem Fall mit genau einem Ctrl-Z zum vorherigen Textzustand zurückkehren können.
- **SC-004**: Selektionen über mehrere Zeilen und an Dokumentgrenzen werden konsistent behandelt, ohne Datenverlust, halbierte Graphem-Cluster oder unbeabsichtigte Änderungen an nicht-selektiertem Text.
- **SC-005**: Automatisierte Tests decken den primären Selektionsfluss, das Zurücknehmen ohne Shift, das Ersetzen durch Eingabe, das Löschen via DEL/BACKSPACE sowie die definierten Grenzfälle (Dokumentgrenzen, Mehrzeiligkeit, CRLF, leere Selektion) ab.

## Assumptions

- Shift-Kombinationen beziehen sich auf die vier Pfeiltasten (Left, Right, Up, Down) als Selektionsbewegungen; Maus-Selektion ist nicht Teil dieser Funktion.
- Das bestehende `History`/`EditStep`-Modell kann so erweitert werden, dass eine "Ersetzen"- oder kombinierte Lösch-/Einfüge-Aktion als ein atomarer Schritt aufgezeichnet wird; die Redo-Clear-Regel (Redo-Stack leeren bei neuer Änderung) bleibt unverändert.
- Die Spaltenbeibehaltung (preferred_column) bei vertikalen Bewegungen gilt auch für Shift+Up/Shift+Down analog zum bestehenden Verhalten bei Up/Down.
- Befehle, die nicht in den Editiermodus fallen (Suche, Speichern, Quit-Prompt), werden nach heutigem Verhalten priorisiert; sie lassen die Selektion unangetastet, es sei denn, der Moduswechsel erfordert dies ausdrücklich.
- Die Selektion arbeitet konsistent auf Zeichenindizes (wie der bestehende Textpuffer) und nicht auf Byte-Offsets oder reinen Graphem-Offsets; die Darstellung bleibt Graphem-bewusst für CRLF- und Multi-Cluster-Fälle.
- Als Rückkehrposition nach einer Löschung wird die Selektionsanfangsposition (der kleinere der Indizes Anchor/Head) definiert; beide Löschbefehle (DEL, BACKSPACE) führen zur gleichen Cursorposition, da die Selektion den löschbaren Bereich bestimmt.
