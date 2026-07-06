# Feature Specification: Tab Support and Auto-Indent

**Feature Branch**: `[010-tab-auto-indent]`

**Created**: 2026-07-06

**Status**: Draft

**Input**: User description: "Tabulator-Unterstützung und Auto-Indent Als User möchte ich die Unterstützung der Tab-Taste und eine sauberen Einrückung haben. Das bedeutet folgendes: - Wenn ich die Tab-Taste drücke, dann soll eine oder zwei Leerzeichen eingefügt werden, abhängig, ob der Cursor auf einer geraden oder ungerade Position steht. D.h steht der Cursor auf Spalte 1, 3, 5 etc. dann wird nur ein Leerzeichen eingefügt. Steht er hingegen auf 0, 2, 4 etc. dann sind es zwei - Wenn ich Enter drücke und damit einen Zeilenumbruch einfüge, soll die nächste Zeile genausoviele Leerzeichen eingerückt sein, wie die Zeile davor - Wenn ich Backspace drücke und vor dem Cursor sind **nur** Leerzeichen, dann soll die Logik wie bei der Tab-Taste laufen, nur umgekehrt. Es sollen also 1 oder 2 Leerzeichen entfernt werden, je nachdem, wie viele Leerzeichen (gerade oder ungerade) davor stehen."

## Clarifications

### Session 2026-07-06

- Q: Wie verhalten sich Tab, Enter und Backspace bei aktiver Auswahl? → A: Bei aktiver Auswahl ersetzen Tab, Enter und Backspace die Auswahl zuerst und wenden ihre normale Logik danach auf den Einfügepunkt an.
- Q: Was bedeutet „Spalte“ für Tab- und Backspace-Logik? → A: Spalte ist die nullbasierte Anzahl der Zeichen links vom Cursor in der aktuellen Zeile; jedes mit einem Schritt eingefügte oder gelöschte Zeichen zählt als 1.
- Q: Wie verhalten sich Undo-Schritte für Tab, Enter und spezielles Backspace? → A: Jede Tab-, Enter- und Backspace-Aktion ist genau ein Undo-Schritt.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Mit Tab zur nächsten Einrückungsstufe springen (Priority: P1)

Als Nutzer möchte ich mit einem Druck auf die Tab-Taste die Einrückung so erweitern, dass der Cursor immer auf der nächsten geraden Spaltenposition landet. Dadurch kann ich Text gleichmäßig einrücken, ohne manuell zählen zu müssen.

**Why this priority**: Das ist die Kernfunktion der gewünschten Tab-Unterstützung. Ohne sie bleibt das Arbeiten mit Einrückungen langsam und fehleranfällig.

**Independent Test**: Einen Cursor auf verschiedenen Spaltenpositionen innerhalb derselben Zeile platzieren und Tab drücken. Der Test ist bestanden, wenn jeweils genau so viele Leerzeichen eingefügt werden, dass der Cursor auf der nächsten geraden Spaltenposition landet.

**Acceptance Scenarios**:

1. **Given** eine leere Zeile und der Cursor steht auf Spalte 0, **When** der Nutzer Tab drückt, **Then** werden genau zwei Leerzeichen eingefügt und der Cursor steht auf Spalte 2.
2. **Given** eine Zeile mit genau einem führenden Leerzeichen und der Cursor steht auf Spalte 1, **When** der Nutzer Tab drückt, **Then** wird genau ein Leerzeichen eingefügt und der Cursor steht auf Spalte 2.
3. **Given** eine Zeile mit zwei führenden Leerzeichen und der Cursor steht auf Spalte 2, **When** der Nutzer Tab drückt, **Then** werden genau zwei Leerzeichen eingefügt und der Cursor steht auf Spalte 4.
4. **Given** der Cursor steht mitten im Text auf einer ungeraden Spaltenposition, **When** der Nutzer Tab drückt, **Then** wird genau ein Leerzeichen eingefügt und der restliche Text bleibt erhalten.

---

### User Story 2 - Neue Zeile mit gleicher Einrückung beginnen (Priority: P1)

Als Nutzer möchte ich beim Drücken von Enter in der neuen Zeile automatisch dieselbe Anzahl führender Leerzeichen erhalten wie in der vorherigen Zeile. Dadurch kann ich mehrere gleich eingerückte Zeilen schnell nacheinander schreiben.

**Why this priority**: Automatische Einrückung spart bei strukturiertem Text viele wiederholte Tastendrücke und sorgt für konsistente Formatierung.

**Independent Test**: Eine Zeile mit führenden Leerzeichen anlegen, den Cursor an verschiedene Positionen in dieser Zeile setzen und Enter drücken. Der Test ist bestanden, wenn die neue Zeile dieselbe Anzahl führender Leerzeichen erhält wie die ursprüngliche Zeile.

**Acceptance Scenarios**:

1. **Given** eine Zeile beginnt mit zwei Leerzeichen gefolgt von Text und der Cursor steht am Zeilenende, **When** der Nutzer Enter drückt, **Then** wird eine neue Zeile eingefügt, die mit genau zwei Leerzeichen beginnt.
2. **Given** eine Zeile beginnt mit vier Leerzeichen gefolgt von Text und der Cursor steht in der Mitte der Zeile, **When** der Nutzer Enter drückt, **Then** wird die Zeile geteilt und die neue Zeile beginnt mit genau vier Leerzeichen.
3. **Given** eine Zeile hat keine führenden Leerzeichen, **When** der Nutzer Enter drückt, **Then** wird eine neue nicht eingerückte Zeile eingefügt.
4. **Given** eine Zeile besteht nur aus führenden Leerzeichen, **When** der Nutzer Enter drückt, **Then** wird eine neue Zeile mit derselben Anzahl führender Leerzeichen erzeugt.

---

### User Story 3 - Mit Backspace sauber ausrücken (Priority: P1)

Als Nutzer möchte ich beim Drücken von Backspace in einer Einrückung aus nur Leerzeichen wieder zur vorherigen Einrückungsstufe zurückspringen. Dadurch kann ich Einrückungen genauso schnell entfernen, wie ich sie mit Tab aufgebaut habe.

**Why this priority**: Einrückung muss in beide Richtungen konsistent funktionieren. Ohne diese Rückwärtslogik bleibt die Bearbeitung von Einrückungen unvollständig.

**Independent Test**: Den Cursor in einer Zeile hinter unterschiedlich vielen führenden Leerzeichen platzieren und Backspace drücken. Der Test ist bestanden, wenn jeweils genau so viele Leerzeichen entfernt werden, dass der Cursor auf die vorherige gerade Spaltenposition zurückkehrt.

**Acceptance Scenarios**:

1. **Given** eine Zeile beginnt mit einem Leerzeichen und der Cursor steht auf Spalte 1, **When** der Nutzer Backspace drückt, **Then** wird genau ein Leerzeichen entfernt und der Cursor steht auf Spalte 0.
2. **Given** eine Zeile beginnt mit zwei Leerzeichen und der Cursor steht auf Spalte 2, **When** der Nutzer Backspace drückt, **Then** werden genau zwei Leerzeichen entfernt und der Cursor steht auf Spalte 0.
3. **Given** eine Zeile beginnt mit drei Leerzeichen und der Cursor steht auf Spalte 3, **When** der Nutzer Backspace drückt, **Then** wird genau ein Leerzeichen entfernt und der Cursor steht auf Spalte 2.
4. **Given** links vom Cursor befindet sich in derselben Zeile mindestens ein Nicht-Leerzeichen, **When** der Nutzer Backspace drückt, **Then** wird das normale Backspace-Verhalten ausgeführt statt einer Einrückungsstufe.

---

### Edge Cases

- Wenn der Cursor am Zeilenanfang auf Spalte 0 steht und Backspace gedrückt wird, darf keine negative Einrückung entstehen und es wird nichts aus der Zeile entfernt.
- Wenn links vom Cursor in der aktuellen Zeile sowohl Leerzeichen als auch andere Zeichen stehen, darf die spezielle Backspace-Logik nicht greifen; es gilt das normale Löschverhalten.
- Wenn Enter in einer Zeile ohne führende Leerzeichen gedrückt wird, darf die neue Zeile nicht künstlich eingerückt werden.
- Wenn die vorherige Zeile mit Tabulatorzeichen oder anderen Nicht-Leerzeichen beginnt, werden nur führende Leerzeichen als Einrückung übernommen; andere Zeichen werden nicht automatisch kopiert.
- Wenn Tab mitten in einer Zeile gedrückt wird, darf nur die Position rechts vom Cursor verschoben werden; bestehender Text links vom Cursor bleibt unverändert.
- Wenn Enter eine Zeile mit führenden Leerzeichen in zwei Teile teilt, muss der Inhalt rechts vom Cursor erhalten bleiben und erst nach der automatisch gesetzten Einrückung folgen.
- Wenn eine nicht-leere Auswahl aktiv ist und Tab, Enter oder Backspace gedrückt wird, muss zuerst genau der ausgewählte Bereich entfernt werden; erst danach darf die jeweilige Einrückungs- oder Zeilenumbruchlogik greifen.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Das System MUSS beim Drücken der Tab-Taste ausschließlich Leerzeichen einfügen und niemals ein Tabulatorzeichen in das Dokument schreiben.
- **FR-002**: Das System MUSS für diese Funktion die aktuelle Spalte als nullbasierte Anzahl der Zeichen links vom Cursor in der aktuellen Zeile bestimmen; jedes mit einem einzelnen Bearbeitungsschritt eingefügte oder gelöschte Zeichen zählt dabei als 1.
- **FR-003**: Das System MUSS bei Tab genau so viele Leerzeichen einfügen, dass der Cursor von seiner aktuellen Spalte auf die nächste gerade Spaltenposition springt.
- **FR-004**: Das System MUSS bei einer aktuellen geraden Spaltenposition zwei Leerzeichen einfügen.
- **FR-005**: Das System MUSS bei einer aktuellen ungeraden Spaltenposition ein Leerzeichen einfügen.
- **FR-006**: Das System MUSS beim Drücken von Enter eine neue Zeile erzeugen, die mit genau derselben Anzahl führender Leerzeichen beginnt wie die unmittelbar vorherige Zeile.
- **FR-007**: Das System MUSS bei Enter den vorhandenen Inhalt rechts vom Cursor erhalten und nach dem Zeilenumbruch in der neuen Zeile hinter der automatisch übernommenen Einrückung platzieren.
- **FR-008**: Das System MUSS beim Drücken von Backspace die spezielle Ausrück-Logik nur dann anwenden, wenn sich vom Zeilenanfang bis direkt vor dem Cursor ausschließlich Leerzeichen befinden.
- **FR-009**: Das System MUSS bei spezieller Backspace-Logik genau so viele Leerzeichen entfernen, dass der Cursor auf die vorherige gerade Spaltenposition zurückspringt.
- **FR-010**: Das System MUSS bei einer ungeraden Anzahl ausschließlich führender Leerzeichen vor dem Cursor genau ein Leerzeichen entfernen.
- **FR-011**: Das System MUSS bei einer geraden Anzahl ausschließlich führender Leerzeichen vor dem Cursor genau zwei Leerzeichen entfernen.
- **FR-012**: Das System MUSS bei Backspace am Zeilenanfang ohne führende Leerzeichen keine Einrückungszeichen entfernen.
- **FR-013**: Das System MUSS außerhalb der beschriebenen Einrückungssituationen das bestehende Standardverhalten von Enter und Backspace unverändert beibehalten.
- **FR-014**: Das System MUSS definieren, dass nur führende Leerzeichen als Einrückung zählen; andere Zeichen am Zeilenanfang gelten nicht als automatische Einrückung.
- **FR-015**: Das System MUSS bei aktiver, nicht-leerer Auswahl für Tab, Enter und Backspace zuerst die Auswahl löschen, danach die jeweilige Tab-, Enter- oder Backspace-Logik einmalig am Einfügepunkt anwenden und die Auswahl aufheben.
- **FR-016**: Das System MUSS die Verantwortlichkeiten für Tasteninterpretation, Textänderung und Cursorplatzierung klar getrennt halten, damit Tab-, Enter- und Backspace-Verhalten unabhängig verständlich und testbar bleiben.
- **FR-017**: Das System MUSS jede einzelne Tab-, Enter- und Backspace-Aktion, einschließlich Auswahlersetzung und spezieller Einrückungslogik, als genau einen Undo-Schritt behandeln.
- **FR-018**: Das System MUSS automatisierte Tests für Tab auf geraden und ungeraden Spalten, Enter mit und ohne führende Leerzeichen sowie Backspace mit reiner Leerzeichen-Einrückung, gemischtem Inhalt vor dem Cursor, aktiver Auswahl und Undo-Verhalten vorsehen.

### Key Entities *(include if feature involves data)*

- **Cursor Column**: Die nullbasierte Anzahl der Zeichen links vom Cursor innerhalb der aktuellen Zeile. Sie bestimmt, ob Tab oder Backspace um eine oder zwei Leerzeichen arbeitet; jedes mit einem einzelnen Bearbeitungsschritt eingefügte oder gelöschte Zeichen zählt als 1.
- **Leading Indentation**: Die zusammenhängende Folge von Leerzeichen am Anfang einer Zeile. Sie wird für Auto-Indent bei Enter übernommen und für spezielle Backspace-Logik ausgewertet.
- **Current Line Content**: Der gesamte Inhalt der bearbeiteten Zeile links und rechts vom Cursor. Er bleibt bei Einrückungsoperationen erhalten, außer dort, wo gezielt Leerzeichen eingefügt oder entfernt werden.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In allen definierten Akzeptanztests für Tab landet der Cursor nach einem Tastendruck immer auf der nächsten geraden Spaltenposition.
- **SC-002**: In allen definierten Akzeptanztests für Enter beginnt die neu erzeugte Zeile mit exakt derselben Anzahl führender Leerzeichen wie die Ausgangszeile.
- **SC-003**: In allen definierten Akzeptanztests für Backspace mit reiner Leerzeichen-Einrückung springt der Cursor nach einem Tastendruck immer auf die vorherige gerade Spaltenposition oder bleibt bei Spalte 0.
- **SC-004**: In allen Tests mit gemischtem Inhalt vor dem Cursor bleibt das normale Backspace-Verhalten erhalten und die spezielle Ausrück-Logik wird nicht fälschlich ausgelöst.
- **SC-005**: In allen Undo-Tests lässt sich jede einzelne Tab-, Enter- und Backspace-Aktion mit genau einem Undo-Schritt vollständig rückgängig machen.
- **SC-006**: Automatisierte Tests decken die Primärflüsse und alle in dieser Spezifikation genannten Edge Cases für Tab, Enter und Backspace ab.

## Assumptions

- Spaltenpositionen werden für diese Funktion als nullbasierte Anzahl der Zeichen links vom Cursor betrachtet, entsprechend den im Nutzerwunsch genannten Beispielen für Spalte 0, 1, 2, 3 und so weiter.
- Die gewünschte Einrückungslogik arbeitet in Schritten von zwei Spalten und nutzt dafür ausschließlich Leerzeichen.
- Die bestehende Enter-Funktion zum Erzeugen eines Zeilenumbruchs bleibt erhalten und wird nur um das automatische Übernehmen führender Leerzeichen ergänzt.
- Die bestehende Backspace-Funktion bleibt für alle Situationen außerhalb reiner Leerzeichen-Einrückung unverändert.
- Dokumente können bereits Tabulatorzeichen enthalten, aber diese Funktion behandelt nur führende Leerzeichen als automatische Einrückung.