# Usage

1. See [here](https://docs.ros.org/en/rolling/Installation.html) to install ROS2 rolling.
2. Build [rmw_zenoh](https://github.com/ros2/rmw_zenoh).
  ```bash
  mkdir ~/ws_rmw_zenoh/src -p && cd ~/ws_rmw_zenoh/src
  git clone https://github.com/ros2/rmw_zenoh.git
  cd ~/ws_rmw_zenoh
  rosdep install --from-paths src --ignore-src --rosdistro rolling
  source /opt/ros/rolling/setup.bash
  colcon build --cmake-args -DCMAKE_BUILD_TYPE=Release
  ```
3. Build the test
  ```bash
  source ./install/setup.bash
  cargo build --release --bins
  ```
4. Run the test
  ```bash
  ./test.sh
  ```
