WITH table_columns AS (
    SELECT 
        table_schema,
        table_name,
        column_name,
        CASE 
            WHEN data_type = 'character varying' THEN 'VARCHAR(' || character_maximum_length || ')'
            WHEN data_type = 'character' THEN 'CHAR(' || character_maximum_length || ')'
            WHEN data_type = 'numeric' THEN 'NUMERIC(' || numeric_precision || ',' || numeric_scale || ')'
            ELSE data_type
        END AS column_definition,
        CASE 
            WHEN is_nullable = 'NO' THEN ' NOT NULL'
            ELSE ''
        END AS null_constraint
    FROM 
        information_schema.columns
    WHERE 
        table_schema NOT IN ('pg_catalog', 'information_schema')
),
foreign_keys AS (
    SELECT 
        kcu.table_schema,
        kcu.table_name,
        kcu.column_name,
        ccu.table_schema AS foreign_table_schema,
        ccu.table_name AS foreign_table_name,
        ccu.column_name AS foreign_column_name
    FROM 
        information_schema.key_column_usage kcu
    JOIN 
        information_schema.referential_constraints rc ON kcu.constraint_name = rc.constraint_name
    JOIN 
        information_schema.constraint_column_usage ccu ON ccu.constraint_name = rc.unique_constraint_name
)
SELECT 
    'CREATE TABLE ' || tc.table_schema || '.' || tc.table_name || ' (' || 
    string_agg(tc.column_name || ' ' || tc.column_definition || tc.null_constraint, ', ') || 
    CASE 
        WHEN COUNT(fk.foreign_table_name) > 0 THEN ', ' || string_agg('FOREIGN KEY (' || fk.column_name || ') REFERENCES ' || fk.foreign_table_schema || '.' || fk.foreign_table_name || '(' || fk.foreign_column_name || ')', ', ')
        ELSE ''
    END || 
    ');' AS create_table_statement
FROM 
    table_columns tc
LEFT JOIN 
    foreign_keys fk ON tc.table_schema = fk.table_schema AND tc.table_name = fk.table_name AND tc.column_name = fk.column_name
GROUP BY 
    tc.table_schema, tc.table_name
ORDER BY 
    tc.table_schema, tc.table_name;
