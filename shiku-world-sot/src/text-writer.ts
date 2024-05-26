export class TextWriter {
  current_line: string = '';
  current_inserted_index: number = 0;
  current_lines: string[] = [];
  last_new_text_write: number = 0;
  max_lines: number = 6;
  max_line_length: number = 50;
  delay: number = 0.01;

  constructor(private text_box: HTMLElement) {}

  set_delay(delay: number) {
    this.delay = delay;
  }

  text_write(text: string) {
    console.log(this.text_box, text);
  }
}
