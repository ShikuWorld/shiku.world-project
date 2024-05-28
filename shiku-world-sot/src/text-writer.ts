export class TextWriter {
  _current_text: string = '';
  current_text_stream_start_index: number = 0;
  current_inserted_index: number = 0;
  current_line_index: number = 0;
  pages: string[] = [];
  current_line_element: HTMLElement = document.createElement('p');
  time_of_last_write = Date.now();
  max_lines: number = 6;
  current_sot_index: number = -1;
  current_page_index: number = 0;
  max_chars_per_line: number = 50;
  add_char_delay_in_ms: number = 50;
  line_delay_in_ms: number = 5000;
  go_to_next_page_time_in_ms: number = 1000;
  clear_idle_time_in_ms: number = 15000;
  current_span_class: string = '';
  private _next_page_timeout: number | null = null;
  private _character_write_timeout: number | null = null;
  private _new_line_timeout: number | null = null;
  private _full_clear_timeout: number | null = null;

  get current_text() {
    return this._current_text;
  }

  set current_text(text: string) {
    this._current_text = text;
    this.pages = this.get_pages_from_text(this.current_text);
    this._update_text_box_chars();
  }

  constructor(private text_box: HTMLElement) {
    this._create_new_line();
  }

  set_delay(delay: number) {
    this.add_char_delay_in_ms = delay;
  }

  write_text(text: string, sot_index: number, span_class: string = '') {
    if (text.trim() === '') {
      return;
    }
    this.current_span_class = span_class;

    this._update_current_text(text, sot_index);
    this.write_next_character();
  }

  write_next_character() {
    const writeChar = () => {
      if (this._next_page_timeout != null) {
        return;
      }
      this._clear_schedules();
      this._schedule_full_clear();
      this._schedule_new_line();
      const page = this.pages[this.current_page_index];
      if (page === undefined) {
        console.error(this.pages, this.current_page_index);
        return;
      }
      if (this.current_inserted_index < page.trim().length) {
        if (
          this.current_inserted_index > 1 &&
          (this.current_inserted_index + 1) % this.max_chars_per_line === 1
        ) {
          this._create_new_line();
        }
        this._add_char_and_schedule_next(writeChar);
      } else {
        if (this.pages[this.current_page_index + 1]) {
          this._clear_schedules();
          this._schedule_next_page();
        }
        return;
      }
    };
    writeChar();
  }

  private _add_char_and_schedule_next(writeChar: () => void) {
    const page = this.pages[this.current_page_index];
    const char_to_write = page[this.current_inserted_index];
    this._add_char_to_line(page[this.current_inserted_index]);
    this.current_inserted_index++;
    const next_char_to_write = page[this.current_inserted_index + 1];
    if (char_to_write === ' ' && next_char_to_write === ' ') {
      writeChar();
      return;
    }

    this._character_write_timeout = setTimeout(
      writeChar,
      this.add_char_delay_in_ms,
    );
  }

  private _update_current_text(text: string, sot_index: number) {
    if (this.current_sot_index === sot_index) {
      this.current_text =
        this.current_text.slice(0, this.current_text_stream_start_index) +
        this._capitalize_first_letter(text.trim());
    } else {
      if (
        this.current_text.length > 0 &&
        !this._text_has_end_marker(this.current_text)
      ) {
        this.current_text += '. ';
      }
      this.current_text_stream_start_index = this.current_text.length;

      this.current_text += this._capitalize_first_letter(text.trim());
    }
    this.current_sot_index = sot_index;
    this.time_of_last_write = Date.now();
  }

  private _capitalize_first_letter(string: string) {
    return string.charAt(0).toUpperCase() + string.slice(1);
  }

  private _update_text_box_chars() {
    const page = this.pages[this.current_page_index];
    if (page === undefined) {
      console.error(this.pages, this.current_page_index);
      return;
    }
    this.text_box.querySelectorAll('span').forEach((span, index) => {
      if (!page[index]) {
        return;
      }
      span.innerHTML = page[index];
    });
  }

  private _schedule_next_page() {
    if (this._next_page_timeout) {
      clearTimeout(this._next_page_timeout);
    }
    this._next_page_timeout = setTimeout(() => {
      this._next_page_timeout = null;
      this.current_page_index++;
      this.current_inserted_index = 0;
      this.current_line_index = 0;
      this.text_box.innerHTML = '';
      this._create_new_line();
      this.write_next_character();
    }, this.go_to_next_page_time_in_ms);
  }

  private _schedule_new_line() {
    if (this._new_line_timeout) {
      clearTimeout(this._new_line_timeout);
    }
    this._new_line_timeout = setTimeout(() => {
      if (this.current_text.length > 0) {
        this.current_text += this._text_has_end_marker(this.current_text)
          ? '\n'
          : '.\n';
      }
      this.current_text_stream_start_index = this.current_text.length;
    }, this.line_delay_in_ms);
  }

  private _text_has_end_marker(text: string) {
    const t = text.endsWith('\n') ? text.slice(0, text.length - 1) : text;
    return t.endsWith('.') || t.endsWith('!') || t.endsWith('?');
  }

  private _create_new_line() {
    if (this.current_line_index < this.max_lines) {
      this.current_line_element = document.createElement('p');
      this.text_box.appendChild(this.current_line_element);
      this.current_line_index += 1;
    }
  }

  private _schedule_full_clear() {
    if (this._full_clear_timeout) {
      clearTimeout(this._full_clear_timeout);
    }
    this._full_clear_timeout = setTimeout(() => {
      this.current_line_index = 0;
      this.current_text_stream_start_index = 0;
      this.current_inserted_index = 0;
      this.current_page_index = 0;
      this.current_text = '';
      this.text_box.innerHTML = '';
      this.pages = [];
    }, this.clear_idle_time_in_ms);
  }

  get_pages_from_text(text: string): string[] {
    const pages: string[] = [];
    let page = '';

    const lines = text.split('\n').filter((l) => l.trim() !== '');
    for (const l of lines) {
      const words = l.split(' ');
      let words_per_line: string[] = [];
      let current_line_length = 0;
      words.forEach((word) => {
        if (current_line_length + word.length > this.max_chars_per_line) {
          const line = words_per_line.join(' ');
          const diff_until_line_full =
            this.max_chars_per_line - current_line_length + 1;
          page += line + ' '.repeat(diff_until_line_full);
          current_line_length = 0;
          words_per_line = [];
        }
        if (page.length >= this.max_chars_per_line * this.max_lines) {
          pages.push(page);
          page = '';
        }
        current_line_length += word.length + 1;
        words_per_line.push(word);
      });
      const diff_until_line_full =
        this.max_chars_per_line - current_line_length + 1;
      page += words_per_line.join(' ') + ' '.repeat(diff_until_line_full);
    }
    pages.push(page);
    return pages;
  }

  private _clear_schedules() {
    [
      this._character_write_timeout,
      this._full_clear_timeout,
      this._new_line_timeout,
      this._next_page_timeout,
    ].forEach((timeout) => {
      if (timeout) {
        clearTimeout(timeout);
        timeout = null;
      }
    });
  }

  private _add_char_to_line(char: string) {
    const span = document.createElement('span');
    span.textContent = char;
    span.className = this.current_span_class || '';
    this.current_line_element.appendChild(span);
  }
}
