import { StyleSheet, View, Text } from 'react-native';
import { helloWorld } from '@bennyhodl/ddk-rn';

export default function App() {
  return (
    <View style={styles.container}>
      <Text>Result: {helloWorld()}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  box: {
    width: 60,
    height: 60,
    marginVertical: 20,
  },
});
