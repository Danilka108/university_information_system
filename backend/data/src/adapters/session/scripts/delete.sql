DELETE
    FROM user_sessions
    WHERE user_id = $1 AND metadata = $2
    RETURNING *;
