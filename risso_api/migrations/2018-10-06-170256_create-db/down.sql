
DROP TABLE comments;
DROP TABLE preferences;
DROP TABLE threads;



SELECT comments.parent,count(*)
    FROM comments INNER JOIN threads ON
        threads.uri=? AND comments.tid=threads.id AND
        (? | comments.mode = ?) AND
        comments.created > ?
    GROUP BY comments.parent
