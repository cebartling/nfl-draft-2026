-- Add chart_type column to draft_sessions table
-- This enables per-session trade value chart selection

ALTER TABLE draft_sessions
ADD COLUMN chart_type VARCHAR(50) NOT NULL DEFAULT 'JimmyJohnson';

-- Add comment for documentation
COMMENT ON COLUMN draft_sessions.chart_type IS
'Trade value chart type to use for this session. Options: JimmyJohnson, RichHill, ChaseStudartAV, FitzgeraldSpielberger, PffWar, SurplusValue';

-- Add check constraint for valid values
ALTER TABLE draft_sessions
ADD CONSTRAINT valid_chart_type CHECK (
    chart_type IN (
        'JimmyJohnson',
        'RichHill',
        'ChaseStudartAV',
        'FitzgeraldSpielberger',
        'PffWar',
        'SurplusValue'
    )
);
