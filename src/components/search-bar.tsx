import { useEffect, useState, useRef } from 'react';
import { Input } from './ui/input';
import { Button } from './ui/button';
import { SearchIcon, XIcon } from 'lucide-react';
import { InputGroup, InputGroupAddon, InputGroupInput } from './ui/input-group';
import { Kbd, KbdGroup } from './ui/kbd';

export default function SearchBar() {
  const [searchQuery, setSearchQuery] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);
  const [focused, setFocused] = useState(false);

  const handleSearchQueryChange = (query: string) => {
    setSearchQuery(query);
    if (query.trim() === '') return;
    console.log('Search query changed:', query);
  };

  // Focus search bar when Ctrl + K is pressed
  useEffect(() => {
    console.log(inputRef.current);

    const handleKeyDown = (e: KeyboardEvent) => {
      const modifierPressed = isMac ? e.metaKey : e.ctrlKey;

      if (modifierPressed && e.key.toLowerCase() === 'k') {
        e.preventDefault();

        inputRef.current?.focus();
        inputRef.current?.select();
      }
    };
    window.addEventListener('keydown', handleKeyDown);

    inputRef.current?.focus();

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);

  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;

  return (
    <div className="p-4">
      <div className="rounded-(--radius) relative flex items-center justify-center">
        <InputGroup className="max-w-lg">
          <InputGroupInput
            type="text"
            placeholder="Search for a file..."
            className="h-full w-full border-none bg-transparent p-2 focus:border-none focus:outline-none transition-shadow transition-duration-100 "
            value={searchQuery}
            onChange={(e) => handleSearchQueryChange(e.target.value)}
            onFocus={() => setFocused(true)}
            onBlur={() => setFocused(false)}

            ref={inputRef}
            autoComplete="off"
            inputMode="search"
            spellCheck={false}
          />
          <InputGroupAddon>
            <SearchIcon />
          </InputGroupAddon>
          <InputGroupAddon align="inline-end" className="relative">
            {searchQuery ? (
              <div className="absolute right-1 top-1/2 -translate-y-1/2">
                <Button
                  variant="secondary"
                  size="icon-xs"
                  className=" active:translate-y-0 data-[slot=button]:active:scale-95 "
                  onClick={() => setSearchQuery('')}
                >
                  <XIcon />
                </Button>
              </div>
            ) : (
              <KbdGroup>
                <Kbd>{isMac ? '⌘' : 'Ctrl'}</Kbd>
                <Kbd>K</Kbd>
              </KbdGroup>
            )}
          </InputGroupAddon>
        </InputGroup>
      </div>
    </div>
  );
}
