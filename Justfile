# inkaartbrenger

# Copyright © Josep Bigorra <jjbigorra@gmail.com>

# inkaartbrenger is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

# inkaartbrenger is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.

# You should have received a copy of the GNU Affero General Public License
# along with inkaartbrenger.  If not, see <https://www.gnu.org/licenses/>.

run:
    guix shell -f guix.scm -- inkaartbrenger --url https://jointhefreeworld.org
    
# run-wikimusic:
#     cargo run -- --url https://wikimusic.jointhefreeworld.org    
# run-byggsteg:
#     cargo run -- --url https://byggsteg.jointhefreeworld.org    
# run-artanis:
#     cargo run -- --url https://artanis.dev
