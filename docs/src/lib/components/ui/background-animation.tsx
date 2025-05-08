import { useEffect, useState, useMemo } from 'react';

interface Point {
  x: number;
  y: number;
}

export function BackgroundAnimation() {
  const [points, setPoints] = useState<Point[]>(() =>
    Array(16).fill(0).map(() => ({
      x: Math.random(),
      y: Math.random()
    }))
  );

  const poly = useMemo(() =>
    points.map(({ x, y }) => `${x * 100}% ${y * 100}%`).join(', '),
    [points]
  );

  const jumpVal = (val: number) => {
    return Math.random() > 0.5 ? val + (Math.random() - 0.5) / 2 : Math.random();
  };

  function jumpPoints() {
    setPoints(prevPoints =>
      prevPoints.map(point => ({
        x: jumpVal(point.x),
        y: jumpVal(point.y)
      }))
    );
  }

  useEffect(() => {
    jumpPoints()
    const timeout = setTimeout(jumpPoints, 2000 + Math.random() * 1000);
    const interval = setInterval(jumpPoints, 3000 + Math.random() * 1000);

    return () => {
      clearTimeout(timeout);
      clearInterval(interval);
    };
  }, []);

  return (
    <div className="bg absolute inset-0 -z-10 transform-gpu blur-3xl overflow-hidden" aria-hidden="true">
      <div
        className="aspect-[1.7] h-full w-full bg-gradient-to-r from-primary to-white/10 lg:opacity-20 lg:dark:opacity-40 xs:opacity-50"
        style={{
          clipPath: `polygon(${poly})`,
          transition: 'clip-path 3s'
        }}
      />
    </div>
  );
} 