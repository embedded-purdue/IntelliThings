// HTTP server: the public demo page (/), the admin page (/admin) and their JSON API.
// Admin calls need the X-Admin-Pin header: a speed bump for visitors, not security.
#pragma once

void web_server_start(void);
